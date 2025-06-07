use std::{
    convert::Infallible,
    ops::{ControlFlow, FromResidual, Try},
};
#[cfg(feature = "trace")]
use tracing::{event, Level};

use crate::RespError;

use super::RespResult;

impl<T, E: RespError> Try for RespResult<T, E> {
    type Output = T;

    type Residual = RespResult<Infallible, E>;

    #[inline]
    fn from_output(output: Self::Output) -> Self {
        Self::Success(output)
    }
    #[inline]
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            RespResult::Success(data) => {
                #[cfg(feature = "trace")]
                event!(Level::TRACE, control_flow = "Continue");
                ControlFlow::Continue(data)
            }
            RespResult::Err(e) => {
                #[cfg(feature = "trace")]
                event!(Level::TRACE, control_flow = "Break");
                ControlFlow::Break(RespResult::Err(e))
            }
        }
    }
}

impl<T, E, Ei> FromResidual<RespResult<Infallible, Ei>> for RespResult<T, E>
where
    E: From<Ei>,
{
    #[inline]
    fn from_residual(residual: RespResult<Infallible, Ei>) -> Self {
        match residual {
            RespResult::Err(e) => Self::Err(From::from(e)),
            RespResult::Success(_) => unreachable!(),
        }
    }
}

impl<T, E, F> FromResidual<Result<Infallible, E>> for RespResult<T, F>
where
    F: From<E>,
{
    #[inline]
    fn from_residual(residual: Result<Infallible, E>) -> Self {
        match residual {
            Err(e) => Self::Err(F::from(e)),
            Ok(_) => unreachable!(),
        }
    }
}

impl<T, E, F> FromResidual<RespResult<Infallible, E>> for Result<T, F>
where
    F: From<E>,
{
    #[inline]
    fn from_residual(residual: RespResult<Infallible, E>) -> Self {
        match residual {
            RespResult::Err(err) => Result::Err(F::from(err)),
            RespResult::Success(_) => unreachable!(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{RespError, RespResult};

    struct A;
    struct B;

    impl From<A> for MockA {
        fn from(a: A) -> Self {
            MockA::A(a)
        }
    }

    impl From<B> for MockA {
        fn from(v: B) -> Self {
            MockA::B(v)
        }
    }

    enum MockA {
        A(A),
        B(B),
    }
    impl RespError for MockA {
        fn log_message(&self) -> std::borrow::Cow<'static, str> {
            "MockA".into()
        }

        #[cfg(feature = "extra-error")]
        type ExtraMessage = String;
        #[cfg(feature = "extra-error")]
        fn extra_message(&self) -> Self::ExtraMessage {
            String::new()
        }
    }

    // test whether ? can work on Result
    fn _testb() -> RespResult<u32, MockA> {
        let a = Result::<_, A>::Ok(11u32)?;
        let _b = RespResult::<_, MockA>::ok(a)?;
        let c = Result::<u32, B>::Err(B)?;

        RespResult::Success(c)
    }
}
