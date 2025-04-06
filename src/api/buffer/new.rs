use zl::{Context, Error, NonYieldable};

pub fn new(_: &mut Context<NonYieldable>) -> Result<(), Error> {
    Ok(())
}
