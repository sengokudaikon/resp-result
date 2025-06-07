mod axum;
pub trait ToInner {
    type Inner;
    fn to_inner(self) -> Self::Inner;
}

pub trait FromRequestFamily<E> {
    type Payload: ToInner;
}

impl<E, F> FromRequestFamily<E> for F
where
    F: ToInner,
{
    type Payload = Self;
}

pub struct MapReject<T: FromRequestFamily<E>, E>(
    pub <T::Payload as ToInner>::Inner,
);
