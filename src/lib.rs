use chrono::Utc;
use log::{LevelFilter, Level, Log, Metadata, Record};

pub struct Seq {
    default_level: LevelFilter,
    ingest_url: String,
    api_key: String,
    application: String,
    module: String
}

impl Seq {
    pub fn new(api_key: &str, ingest_url: &str, application: &str, module: &str) -> Self {
        Seq {
            default_level: LevelFilter::Info,
            ingest_url: ingest_url.to_string(),
            api_key: api_key.to_string(),
            application: application.to_string(),
            module: module.to_string(),
        }
    }

    pub fn init(self) {
        log::set_max_level(self.default_level);
        log::set_boxed_logger(Box::new(self)).expect("Unable to set seq as a logger");
    }

    fn level_to_seq_level(level: &Level) -> String {
        match level {
            Level::Trace => String::from("Verbose"),
            Level::Debug => String::from("Debug"),
            Level::Info => String::from("Information"),
            Level::Warn => String::from("Warning"),
            Level::Error => String::from("Error"),
        }
    }

    fn debug_print(record: &Record) {
        let prefix = match record.level() {
            Level::Trace => "[ TRACE ]",
            Level::Debug => "[ DEBUG ]",
            Level::Info => "[ INFO ]",
            Level::Warn => "[ WARN ]",
            Level::Error => "[ ERROR ]"
        };
        println!("{} {}", prefix, record.args().to_string().replace("\"", ""));
    }
}

impl Log for Seq {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level().to_level_filter() <= self.default_level
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        // Ugly workaround for ureq causing an infinite loop if the user would use trace as a level
        // if record.module_path().unwrap_or("").contains("ureq") {
        //     return;
        // }
        if !record.module_path().unwrap_or("").contains(self.module.as_str()) && !(record.metadata().level().to_level_filter() <= LevelFilter::Warn) {
            return;
        }

        Seq::debug_print(&record);

        let msgs = format!(
            "{{\"@t\": \"{}\", \"@mt\": \"{}\", \"@l\": \"{}\", \"Application\": \"{}\", \"line\": \"{}\", \"module\": \"{}\", \"file\": \"{}\"}}",
            Utc::now().format("%+"),
            record.args().to_string().replace("\"", ""),
            Seq::level_to_seq_level(&record.level()),
            self.application,
            record.line().unwrap_or(0),
            record.module_path().unwrap_or(""),
            record.file().unwrap_or("")
        );
        let ingest_url = format!("{}/api/events/raw?clef", self.ingest_url);
        match ureq::post(ingest_url.as_str())
            .set("X-Seq-ApiKey", &self.api_key)
            .set("Content-Type", "application/vnd.serilog.clef")
            .send_string(msgs.as_str()) {
            Ok(_) => {},
            Err(why) => {
                eprintln!("Updating seq logs failed: {:?}", why);
            }
        }
    }

    fn flush(&self) {

    }
}

#[cfg(test)]
mod test {
    use super::Seq;

    #[test]
    fn basics() {
        Seq::new("", "", "log_seq test", "log_seq").init();
        log::warn!("test test");
        log::error!("Testing an error code");
    }
}
