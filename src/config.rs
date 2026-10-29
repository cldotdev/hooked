use clap::ArgAction;
use clap::{Parser, Subcommand};
use std::fs;

#[derive(Parser)]
#[command(version, about, long_about = None, disable_help_flag = true)]
pub struct Config {
    /// Path to the log file.
    #[arg(short, long = "log", env, default_value = "log/app.log")]
    pub log_file_path: String,

    /// The IP address where the server is hosted.
    #[arg(short, long, env = "HOOKED_HOST", default_value = "0.0.0.0")]
    pub host: String,

    /// The port number where the server is listening.
    #[arg(short, long, env = "HOOKED_PORT", default_value_t = 3000)]
    pub port: u16,

    /// Path to the build entry script.
    #[arg(long, env)]
    pub build_entry_script_path: String,

    /// The secret token to verify incoming GitHub or Gitea webhook messages.
    /// See [Creating webhooks - GitHub
    /// Docs](https://docs.github.com/en/webhooks/using-webhooks/creating-webhooks)
    /// or [Webhooks - Gitea
    /// Docs](https://docs.gitea.com/usage/webhooks) for creating a webhook and
    /// setting the secret token.
    #[arg(long, env, value_parser = parse_secret_or_env)]
    pub webhook_secret: String,

    /// The Git branch name to watch.
    #[arg(long, env)]
    pub watch_push_branch: String,

    /// The Discord webhook URL to send the notification to Discord
    /// channel. See [Intro to Webhooks –
    /// Discord](https://support.discord.com/hc/en-us/articles/228383668-Intro-to-Webhooks)
    /// for creating a Discord webhook URL.
    #[arg(long, env, value_parser = parse_secret_or_env)]
    pub discord_webhook_url: Option<String>,

    #[arg(short = '?', long, global = true, action = ArgAction::Help, help = "Print help")]
    pub help: (),

    /// Start the server when omitted.
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Build a commit through the running server, as if it were pushed to the
    /// watched branch.
    Trigger {
        /// The full or abbreviated commit ID, in lowercase hex.
        commit: String,
    },
}

const SECRETS_DIR: &str = "/run/secrets/";

fn parse_secret_or_env(value: &str) -> Result<String, String> {
    parse_secret_in(value, SECRETS_DIR)
}

fn parse_secret_in(value: &str, secrets_dir: &str) -> Result<String, String> {
    if value.starts_with(secrets_dir) {
        fs::read_to_string(value)
            .map(|s| s.trim().to_string())
            .map_err(|e| format!("Failed to read secret file: {}", e))
    } else {
        Ok(value.to_string())
    }
}

impl Config {
    pub fn load() -> Self {
        Self::parse().normalized()
    }

    // clap passes an empty flag or env value through as `Some("")`.
    fn normalized(mut self) -> Self {
        self.discord_webhook_url = self.discord_webhook_url.filter(|u| !u.is_empty());
        self
    }

    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(extra: &[&str]) -> Config {
        let mut args = vec![
            "hooked",
            "--build-entry-script-path=b.sh",
            "--webhook-secret=s",
            "--watch-push-branch=master",
        ];
        args.extend_from_slice(extra);
        Config::try_parse_from(args).unwrap().normalized()
    }

    // Env vars are process-wide, so every case that touches them lives in one test.
    #[test]
    fn discord_url_from_env_and_flag() {
        std::env::remove_var("DISCORD_WEBHOOK_URL");
        assert_eq!(parse(&[]).discord_webhook_url, None);

        std::env::set_var("DISCORD_WEBHOOK_URL", "");
        assert_eq!(parse(&[]).discord_webhook_url, None);
        std::env::remove_var("DISCORD_WEBHOOK_URL");

        let url = "https://discord.com/api/webhooks/1/x";
        assert_eq!(
            parse(&[&format!("--discord-webhook-url={}", url)]).discord_webhook_url,
            Some(url.to_string())
        );
    }

    #[test]
    fn trigger_subcommand_is_optional() {
        assert!(parse(&[]).command.is_none());
        let config = parse(&["trigger", "abc123"]);
        assert!(matches!(config.command, Some(Command::Trigger { commit }) if commit == "abc123"));
    }

    #[test]
    fn new_flag_names_parse() {
        let config = parse(&[]);
        assert_eq!(config.webhook_secret, "s");
        assert_eq!(config.watch_push_branch, "master");
    }

    #[test]
    fn former_flag_names_are_rejected() {
        for old in [
            "--github-webhook-secret=s",
            "--github-watch-push-branch=master",
        ] {
            let args = [
                "hooked",
                "--build-entry-script-path=b.sh",
                "--webhook-secret=s",
                "--watch-push-branch=master",
                old,
            ];
            assert!(Config::try_parse_from(args).is_err(), "{old} accepted");
        }
    }

    #[test]
    fn secret_path_is_read_and_trimmed() {
        let dir = std::env::temp_dir().join(format!("hooked-secrets-{}/", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let dir_str = dir.to_str().unwrap();
        let path = format!("{}url", dir_str);
        std::fs::write(&path, "https://example.com/hook\n").unwrap();

        assert_eq!(
            parse_secret_in(&path, dir_str).unwrap(),
            "https://example.com/hook"
        );
        assert!(parse_secret_in(&format!("{}missing", dir_str), dir_str).is_err());
        assert_eq!(parse_secret_in("literal", dir_str).unwrap(), "literal");
        let _ = std::fs::remove_dir_all(dir);
    }
}
