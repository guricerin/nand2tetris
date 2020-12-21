pub enum CommandType {
    /// address
    /// ex: @Xxx (symbol or numerics)
    Addr,
    /// compute
    /// ex: dest=comp;jump
    Comp,
    /// label (pseudo command)
    /// ex: (Xxx)
    Label,
}
