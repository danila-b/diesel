use std::error::Error;
use std::fmt;
use std::io::Write;

use backend::Backend;
use expression::*;
use expression::bound::Bound;
use query_source::Queryable;
use types::{HasSqlType, FromSql, FromSqlRow, Nullable, ToSql, IsNull, NotNull};

impl<T, DB> HasSqlType<Nullable<T>> for DB where
    DB: Backend + HasSqlType<T>, T: NotNull,
{
    fn metadata() -> DB::TypeMetadata{
        <DB as HasSqlType<T>>::metadata()
    }
}

impl<T, ST, DB> FromSql<Nullable<ST>, DB> for Option<T> where
    T: FromSql<ST, DB>,
    DB: Backend + HasSqlType<ST>, ST: NotNull,
{
    fn from_sql(bytes: Option<&[u8]>) -> Result<Self, Box<Error>> {
        match bytes {
            Some(_) => T::from_sql(bytes).map(Some),
            None => Ok(None)
        }
    }
}

impl<T, ST, DB> Queryable<Nullable<ST>, DB> for Option<T> where
    T: Queryable<ST, DB>,
    DB: Backend + HasSqlType<ST>,
    Option<T::Row>: FromSqlRow<Nullable<ST>, DB>,
    ST: NotNull,
{
    type Row = Option<T::Row>;

    fn build(row: Self::Row) -> Self {
        row.map(T::build)
    }
}

impl<T, ST, DB> ToSql<Nullable<ST>, DB> for Option<T> where
    T: ToSql<ST, DB>,
    DB: Backend + HasSqlType<ST>,
    ST: NotNull,
{
    fn to_sql<W: Write>(&self, out: &mut W) -> Result<IsNull, Box<Error>> {
        if let &Some(ref value) = self {
            value.to_sql(out)
        } else {
            Ok(IsNull::Yes)
        }
    }
}

impl<T, ST> AsExpression<Nullable<ST>> for Option<T> where
    ST: NotNull,
{
    type Expression = Bound<Nullable<ST>, Self>;

    fn as_expression(self) -> Self::Expression {
        Bound::new(self)
    }
}

impl<'a, T, ST> AsExpression<Nullable<ST>> for &'a Option<T> where
    ST: NotNull,
{
    type Expression = Bound<Nullable<ST>, Self>;

    fn as_expression(self) -> Self::Expression {
        Bound::new(self)
    }
}

#[derive(Debug)]
pub struct UnexpectedNullError {
    pub msg: String,
}

impl fmt::Display for UnexpectedNullError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl Error for UnexpectedNullError {
    fn description(&self) -> &str {
        &self.msg
    }
}

#[cfg(test)]
use types;
#[cfg(test)]
use backend::Pg;

#[test]
fn option_to_sql() {
    type Type = types::Nullable<types::VarChar>;
    let mut bytes = Vec::<u8>::new();

    let is_null = ToSql::<Type, Pg>::to_sql(&None::<String>, &mut bytes).unwrap();
    assert_eq!(IsNull::Yes, is_null);
    assert!(bytes.is_empty());

    let is_null = ToSql::<Type, Pg>::to_sql(&Some(""), &mut bytes).unwrap();
    assert_eq!(IsNull::No, is_null);
    assert!(bytes.is_empty());

    let is_null = ToSql::<Type, Pg>::to_sql(&Some("Sean"), &mut bytes).unwrap();
    let expectd_bytes: Vec<_> = "Sean".as_bytes().into();
    assert_eq!(IsNull::No, is_null);
    assert_eq!(expectd_bytes, bytes);
}
