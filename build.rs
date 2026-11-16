fn main() {
    #[cfg(windows)]
    {
        let mut resource = winresource::WindowsResource::new();
        resource.set_icon("icon.ico");
        if let Err(error) = resource.compile() {
            eprintln!("failed to embed the executable icon: {error}");
            std::process::exit(1);
        }
    }
}
