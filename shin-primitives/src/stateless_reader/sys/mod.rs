cfg_if::cfg_if! {
    if #[cfg(unix)] {
        mod unix;
        pub use unix::StatelessFileImpl;
    } else if #[cfg(target_os = "windows")] {
        mod windows;
        pub use windows::StatelessFileImpl;
    } else {
        mod fallback;
        pub use fallback::StatelessFileImpl;
    }
}
