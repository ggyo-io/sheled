pub use derive_entity::Entity;
/// EM trait - Entity Model generated at compile time, implemented by
/// the `drive-entity` macro for any struct.
/// Zero Cost Abstraction ;)
///
/// ```
///  #[derive(Entity)]
///  pub struct User {
///     pub id: i64,
///     pub name: String,
///     pub email: String,
///     pub hash: String,
///  }
/// ```
///
/// Example generated ( via cargo expand ) implementation
/// ```
///            impl ::entity::EM for User {
///            fn entity() -> &'static str {
///                "users"
///            }
///            fn columns() -> &'static [&'static str] {
///                &["id", "name", "email", "hash"]
///            }
///            fn tys() -> &'static [&'static str] {
///                &["i64", "String", "String", "String"]
///            }
///        }
/// ```
pub trait EM {
    fn entity() -> &'static str;
    fn columns() ->  &'static [&'static str];
    fn tys() -> &'static [&'static str];
}
