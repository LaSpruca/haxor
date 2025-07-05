use lazy_static::lazy_static;
use tera::{Context, Tera};

const ROOT_PASSWORD_SECRET_STR: &'static str = include_str!("./objects/root-password-secret.yaml");
const SERVICE_STR: &'static str = include_str!("./objects/service.yaml");
const STATEFUL_SET_STR: &'static str = include_str!("./objects/statefulset.yaml");

lazy_static! {
    static ref TERA: Tera = {
        let mut tera = Tera::default();
        tera.add_raw_templates(vec![
            ("root_password_secret", ROOT_PASSWORD_SECRET_STR),
            ("service", SERVICE_STR),
            ("stateful_set", STATEFUL_SET_STR),
        ])
        .expect("Invalid templates");
        tera
    };
}

#[inline]
pub fn root_password_secret(password: &str) -> Result<String, tera::Error> {
    let mut context = Context::new();
    context.insert("password", password);
    TERA.render("secret", &context)
}

#[inline]
pub fn service() -> Result<String, tera::Error> {
    let context = Context::new();
    TERA.render("service", &context)
}

#[inline]
pub fn stateful_set() -> Result<String, tera::Error> {
    let context = Context::new();
    TERA.render("stateful_set", &context)
}
