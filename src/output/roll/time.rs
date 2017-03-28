use output::roll::Strategy;
use output::file::File;

pub struct TimeStrategy{

}

impl Strategy for TimeStrategy{
    fn enable(&self,file: &File) -> bool {
        true
    }
}
