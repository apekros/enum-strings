use enum_strings::EnumStrings;

#[derive(Clone, Copy, EnumStrings)]
pub enum Route {
    #[strings(path = "/", label = "Home")]
    Home,
    #[strings(label = "Docs", path = "/docs")]
    Docs,
}

const HOME: &str = Route::Home.path();

fn main() {
    assert_eq!(HOME, "/");
    assert_eq!(Route::Docs.path(), "/docs");
    assert_eq!(Route::Docs.label(), "Docs");
}
