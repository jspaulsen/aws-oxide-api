pub enum Outcome<S> {
    Response(S),
    Forward,
}
