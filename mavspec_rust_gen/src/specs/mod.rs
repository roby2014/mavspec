pub(crate) mod dialects;

use crate::generator::GeneratorParams;

pub(crate) trait Spec {
    fn params(&self) -> &GeneratorParams;
}
