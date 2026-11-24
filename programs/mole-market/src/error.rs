use anchor_lang::prelude::*;

#[error_code]
pub enum MoleError {
    #[msg("Seat index out of range (0-7)")]
    SeatOutOfRange,
    #[msg("Seat is already taken")]
    SeatTaken,
    #[msg("Seat is empty")]
    SeatEmpty,
    #[msg("Game is not in the lobby phase")]
    NotInLobby,
    #[msg("Game is not in the trading phase")]
    NotTrading,
    #[msg("Game is not in the revealed phase")]
    NotRevealed,
    #[msg("Not all 8 seats are filled")]
    SeatsNotFull,
    #[msg("Signer is not the seat owner or an authorized session key")]
    NotSeatOwner,
    #[msg("Trade amount must be greater than zero")]
    AmountZero,
    #[msg("Insufficient balance for this trade")]
    InsufficientBalance,
    #[msg("Session key has expired")]
    SessionExpired,
    #[msg("A VRF request is already pending for this game")]
    VrfAlreadyPending,
    #[msg("No VRF request is pending")]
    VrfNotPending,
    #[msg("This VRF callback is for a superseded request")]
    StaleCallback,
    #[msg("Moles have already been picked for this round")]
    MolesAlreadyPicked,
    #[msg("VRF request has not timed out yet; wait before using demo randomness")]
    VrfNotTimedOut,
    #[msg("Only the game creator can call this")]
    NotCreator,
    #[msg("Only bot seats can use the bot trade instruction")]
    NotABot,
    #[msg("Arithmetic overflow")]
    MathOverflow,
}
