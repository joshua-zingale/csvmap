pub enum Either<A, B> {
    A(A),
    B(B),
}

impl<A, B> std::io::Read for Either<A, B>
where
    A: std::io::Read,
    B: std::io::Read,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Either::A(a) => a.read(buf),
            Either::B(b) => b.read(buf),
        }
    }
}
