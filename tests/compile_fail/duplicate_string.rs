use enum_strings::EnumStrings;

#[derive(EnumStrings)]
enum Route {
    #[strings(path = "/", path = "/again")]
    Home,
}

fn main() {}
