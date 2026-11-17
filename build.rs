fn main() {
    println!("cargo:rerun-if-changed=assets/app.ico");
    if std::path::Path::new("assets/app.ico").exists() {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/app.ico");
        if let Err(e) = res.compile() {
            panic!("Не удалось встроить иконку приложения: {e}");
        }
    }
}
