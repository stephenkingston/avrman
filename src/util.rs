use indicatif::{ProgressBar, ProgressStyle};

pub(crate) fn create_progress_bar(total_steps: u64, msg: &str) -> ProgressBar {
    let pb = ProgressBar::new(total_steps);

    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "[{spinner:.green} {elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} ({percent}%) {msg}",
            )
            .expect("Failed to create progress bar")
            .progress_chars("#>-"),
    );
    pb.set_message(msg.to_owned());

    pb
}
