use clap::Command;
pub fn command() -> Command {
    Command::new("cli")
        .version("0.1.0") //TODO: handle this later
        .about("runnig our cronjobs using cli")
}
