use anyhow::Result;

pub trait Battery: Send {
    fn update(&mut self) -> Result<()>;
    fn percentage(&self) -> i32;
    fn charging(&self) -> bool;
}

impl Battery for Box<dyn Battery> {
    fn update(&mut self) -> Result<()> {
        (**self).update()
    }

    fn percentage(&self) -> i32 {
        (**self).percentage()
    }

    fn charging(&self) -> bool {
        (**self).charging()
    }
}
