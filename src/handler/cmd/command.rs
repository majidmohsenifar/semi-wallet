use clap::Command;
pub fn command() -> Command {
    Command::new("cli").about("runnig our cronjobs using cli")
}
