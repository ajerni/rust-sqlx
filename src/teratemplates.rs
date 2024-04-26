use actix_web::{get, Responder, HttpResponse};
use lazy_static::lazy_static;
use tera::Tera;

//https://gitlab.com/codescope-reference/rustmx

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let source = "templates/**/*";
        let tera = Tera::new(source).unwrap();
        tera
    };
}

#[get("/teratemplates")]
pub async fn teratemplates() -> impl Responder {
    let mut context = tera::Context::new();
    context.insert("message_from_rust", "Hello from Rust!");
    let page_content = TEMPLATES.render("index.html", &context).unwrap();
    HttpResponse::Ok().body(page_content)
}

#[get("/edit")]
pub async fn edit() -> impl Responder {
    //let word_pairs = database::get_all_wordpairs();
    let mut context = tera::Context::new();
    //context.insert("word_pairs", &word_pairs);
    context.insert("my_content", "guguseli");
    context.insert("my_content2", "dada");
    let page_content = TEMPLATES.render("edit.html", &context).unwrap();
    HttpResponse::Ok().body(page_content)
}


