#![windows_subsystem = "windows"]

mod load_handler;

use std::thread;

use sciter;
use sciter::dispatch_script_call;
use sciter::make_args;
use sciter::Value;
use verifier;

struct EventHandler {
    frontend_logger: Option<sciter::Value>,
    progress_updater: Option<sciter::Value>,
}

impl EventHandler {

    fn get_logger_fn(&self) -> impl Fn(&verifier::VerifyErr) {
        let frontend_logger = self.frontend_logger.as_ref().expect("set_logger() should have been called first!");
        let logger = frontend_logger.clone();

        let progress_updater = self.progress_updater.as_ref().expect("set_progress_updater() should have been called first!");
        let updater = progress_updater.clone();

        move |verify_error: &verifier::VerifyErr| {
            // TODO: error handling
            let _ = logger.call(None, &make_args!(verify_error.message.clone()), None);
            let _ = updater.call(None, &make_args!(33.33), None);
        }
    }

    fn set_progress_updater(&mut self, progress_updater: sciter::Value) {
        self.progress_updater = Some(progress_updater);
    }

    fn set_logger(&mut self, logger: sciter::Value) {
        self.frontend_logger = Some(logger);
    }

    fn verify(&self, src_path: String, dst_path: String) {
        let logger = self.get_logger_fn();
        let stats = verifier::verify_path_with_filtered_logger(&src_path, &dst_path, logger, verifier::verify_err_only_errors_filter);

        let frontend_logger = self.frontend_logger.as_ref().expect("set_logger() should have been called first!");
        let stats_message = format!("{} are OK, {} are missing, {} have file size mismatch!",
                                    stats.get(&verifier::VerifyErrType::OK).unwrap_or(&0),
                                    stats.get(&verifier::VerifyErrType::SrcMissing).unwrap_or(&0) + stats.get(&verifier::VerifyErrType::DstMissing).unwrap_or(&0),
                                    stats.get(&verifier::VerifyErrType::SrcSmaller).unwrap_or(&0) + stats.get(&verifier::VerifyErrType::DstSmaller).unwrap_or(&0));

        // TODO: error handling
        let _ = frontend_logger.call(None, &make_args!(""), None);
        let _ = frontend_logger.call(None, &make_args!("Verification is complete!"), None);
        let _ = frontend_logger.call(None, &make_args!(stats_message), None);
    }

}

impl sciter::EventHandler for EventHandler {

    dispatch_script_call! {
        fn set_logger(Value);
        fn set_progress_updater(Value);
        fn verify(String, String);
    }

}

fn main() {
    let _ = sciter::set_options(
        sciter::RuntimeOptions::ScriptFeatures(sciter::SCRIPT_RUNTIME_FEATURES::ALLOW_SYSINFO as u8)
    );
    let _ = sciter::set_options(sciter::RuntimeOptions::DebugMode(true));

    let resources = include_bytes!("resources.rc");
    let handler = load_handler::LoadHandler::new(resources);
    let event_handler = EventHandler{
        frontend_logger: None,
        progress_updater: None,
    };

    let mut frame = sciter::window::Builder::main_window()
        .with_size((700, 750))
        .create();

    frame.sciter_handler(handler);
    frame.event_handler(event_handler);
    frame.load_file("this://app/html/index.htm");

    frame.run_app();
}

