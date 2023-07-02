fn main() {
    // Enumerate files in sub-folder "res/*", being relevant for the test-generation
    // If function returns with error, exit with error message.
    build_deps::rerun_if_changed_paths("test_data/*/*").unwrap();

    // Adding the parent directory "res" to the watch-list will capture new-files being added
    build_deps::rerun_if_changed_paths("test_data/*").unwrap();
}
