use aardwolf_models::user::email::EmailVerificationToken;
// use ring::rand::SystemRandom;
use rocket::http::{Cookie, Cookies};
use rocket::response::Redirect;
use rocket::request::Form;
use rocket_contrib::Template;

use DbConn;
use forms::auth::{SignInForm, SignUpForm};
use types::SignedInUser;

#[derive(FromForm)]
struct SignUpError {
    msg: String,
}

#[get("/auth/sign_up?<error>")]
fn sign_up_form_with_error(error: SignUpError) -> Template {
    let token = "some csrf token";
    Template::render(
        "sign_up",
        hashmap!{ "token" => token, "error_msg" => error.msg.as_str() },
    )
}

#[get("/auth/sign_up")]
fn sign_up_form() -> Template {
    let token = "some csrf token";
    Template::render("sign_up", hashmap!{ "token" => token })
}

#[derive(FromForm)]
struct SignInError {
    msg: String,
}

#[get("/auth/sign_in?<error>")]
fn sign_in_form_with_error(error: SignInError) -> Template {
    let token = "some csrf token";
    Template::render(
        "sign_in",
        hashmap!{ "token" => token, "error_msg" => error.msg.as_str() },
    )
}

#[get("/auth/sign_in")]
fn sign_in_form() -> Template {
    let token = "some csrf token";
    Template::render("sign_in", hashmap!{ "token" => token })
}

#[post("/auth", data = "<form>")]
fn sign_up(form: Form<SignUpForm>, db: DbConn) -> Redirect {
    use controllers::auth;

    match auth::create_user_and_account(form.into_inner(), &db) {
        Ok(_) => Redirect::to("/auth/sign_in"),
        Err(e) => {
            println!("unable to create account: {:#?}", e);
            Redirect::to(&format!("/auth/sign_up?msg={}", e))
        }
    }
}

#[post("/auth/sign_in", data = "<form>")]
fn sign_in(form: Form<SignInForm>, db: DbConn, mut cookies: Cookies) -> Redirect {
    use controllers::auth;
    use aardwolf_models::user::UserLike;

    match auth::sign_in(form.into_inner(), &db) {
        Ok(user) => {
            let mut cookie = Cookie::new("user_id", format!("{}", user.id()));
            cookie.set_http_only(true);
            cookies.add_private(cookie);
            Redirect::to("/")
        }
        Err(e) => {
            println!("unable to log in: {:#?}", e);
            Redirect::to(&format!("/auth/sign_in?msg={}", e))
        }
    }
}

#[derive(FromForm)]
struct ConfirmToken {
    pub id: i32,
    pub token: EmailVerificationToken,
}

#[derive(Debug, Fail)]
#[fail(display = "Failed to confirm account")]
struct ConfirmError;

#[get("/auth/confirmation?<token>")]
fn confirm(token: ConfirmToken, db: DbConn) -> Result<Redirect, ConfirmError> {
    use controllers::auth;

    Ok(match auth::confirm_account(token.id, token.token, &db) {
        Ok(_) => Redirect::to("/auth/sign_in"),
        Err(e) => {
            println!("unable to confirm account: {:#?}", e);
            return Err(ConfirmError);
        }
    })
}

#[post("/auth/sign_out")]
fn sign_out(_user: SignedInUser, mut cookies: Cookies) -> Redirect {
    cookies.remove_private(Cookie::named("user_id"));
    Redirect::to("/auth/sign_in")
}
