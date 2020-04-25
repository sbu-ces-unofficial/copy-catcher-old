#![windows_subsystem = "windows"]

mod load_handler;

use std::cmp;
use std::thread;

use debug::debug;
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

    fn get_logger_fn(&self, src_path: &str) -> impl FnMut(&verifier::VerifyErr) {
        debug!("get_logger_fn: getting frontend logger...");
        let logger = self.frontend_logger.as_ref().cloned().expect("set_logger() should have been called first!");
        debug!("get_logger_fn: got frontend_logger!");

        debug!("get_logger_fn: getting progress updater...");
        let updater = self.progress_updater.as_ref().cloned().expect("set_progress_updater() should have been called first!");
        debug!("get_logger_fn: got progress updater!");

        // TODO: error handling
        let _ = logger.call(None, &make_args!("Calculating how many files to verify..."), None);
        let num_files = verifier::files::count(src_path);
        let _ = logger.call(None, &make_args!(format!("Verifying {} files...", num_files)), None);
        let _ = logger.call(None, &make_args!(""), None);

        let update_frequency = cmp::max(1, num_files / 100);
        let progress = make_args!((100.0 / num_files as f64).max(1.0_f64));
        debug!(&update_frequency);
        debug!(&progress);
        let mut current_times_called = 0;
        move |verify_error: &verifier::VerifyErr| {
            // TODO: error handling
            if verify_error.kind != verifier::VerifyErrType::OK {
                debug!(format!("get_logger_fn: logging {:?}!", verify_error));
                let _ = logger.call(None, &make_args!(verify_error.message.clone()), None);
            }

            current_times_called += 1;
            if current_times_called > update_frequency {
                debug!("get_logger_fn: updating progress...");
                current_times_called = 0;
                let _ = updater.call(None, &progress, None);
            }
        }
    }

    fn set_progress_updater(&mut self, progress_updater: sciter::Value) {
        self.progress_updater = Some(progress_updater);
    }

    fn set_logger(&mut self, logger: sciter::Value) {
        self.frontend_logger = Some(logger);
    }

    fn verify(&self, src_path: String, dst_path: String) {
        debug!("verify: getting frontend logger...");
        let frontend_logger = self.frontend_logger.as_ref().cloned().expect("set_logger() should have been called first!");
        debug!("verify: got frontend logger!");
        let logger = self.get_logger_fn(&src_path);

        debug!("get_logger_fn: getting progress updater...");
        let updater = self.progress_updater.as_ref().cloned().expect("set_progress_updater() should have been called first!");
        debug!("get_logger_fn: got progress updater!");

        thread::spawn(move || {
            debug!(format!("thread::spawn: starting to verify {}...", &src_path));
            let stats = verifier::verify_path_with_logger(&src_path, &dst_path, logger);

            let stats_message = format!("{} are OK, {} are missing, {} have file size mismatch!",
                                        stats.get(&verifier::VerifyErrType::OK).unwrap_or(&0),
                                        stats.get(&verifier::VerifyErrType::SrcMissing).unwrap_or(&0) + stats.get(&verifier::VerifyErrType::DstMissing).unwrap_or(&0),
                                        stats.get(&verifier::VerifyErrType::SrcSmaller).unwrap_or(&0) + stats.get(&verifier::VerifyErrType::DstSmaller).unwrap_or(&0));

            // TODO: error handling
            let _ = updater.call(None, &make_args!(100), None);

            // TODO: error handling
            let _ = frontend_logger.call(None, &make_args!(""), None);
            let _ = frontend_logger.call(None, &make_args!("Verification is complete!"), None);
            let _ = frontend_logger.call(None, &make_args!(stats_message), None);
        });
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

