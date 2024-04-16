use std::{convert, ops::FromResidual};

use alohomora::rocket::{BBoxRequest, BBoxResponder, BBoxResponseResult};


pub enum MyResult<T, E> {
    Ok(T),
    Err(E)
}

impl<T, E, F: From<E>> FromResidual<Result<convert::Infallible, E>> for MyResult<T, F> {
    fn from_residual(residual: Result<convert::Infallible, E>) -> Self {
        match residual {
            Err(e) => MyResult::Err(From::from(e)),
            _ => unreachable!(),
        }
    }
}

impl<'a, 'r, 'o: 'a, T: BBoxResponder<'a, 'r, 'o>> BBoxResponder<'a, 'r, 'o> for MyResult<T, (rocket::http::Status, String)> {
    fn respond_to(self, request: BBoxRequest<'a, 'r>) -> BBoxResponseResult<'o> {
        match self {
            MyResult::Ok(o) => o.respond_to(request),
            MyResult::Err(e) => Err(e.0),
        }
    }
}