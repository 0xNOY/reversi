pub mod reversi;

use reversi::*;

fn main() {
    let game = &mut Game::new(CliPlayer::default(), WeekBot);
    let finaly_ctx = game.start();
    let (black_num, white_num) = finaly_ctx.board.count_stones();

    println!(
        "{}\n\"{}\" wins in round {}.\n\"{}\": {}, \"{}\": {}",
        CliPlayer::format_board(finaly_ctx),
        Stone::from(finaly_ctx.round - 1),
        finaly_ctx.round,
        Stone::Black,
        black_num,
        Stone::White,
        white_num
    )
}
