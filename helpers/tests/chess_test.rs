//TODO: invalid moves

pub mod tests {
    use helpers::{TicTacToe, chesstactoe::chess::EndResult, ChessBoard};

    const COORDS: (usize, usize) = (0, 0);

    fn test_valid_move(alg: &str, res_fen: &str, tic: &mut TicTacToe) {
        println!("{alg}");

        match tic.validate_move(COORDS, alg) {
            Ok(is_valid) => assert!(is_valid),
            Err(e) => panic!("Unexpected error {e}")
        }
        
        tic.make_move(COORDS, alg).unwrap();

        assert_eq!(tic.get_board(COORDS).unwrap().to_fen(tic.next).unwrap(), res_fen);
    }


    #[test]
    fn kasparov_karpov() {

        let moves: Vec<(&str, &str)> = include_str!("Kasparov_Karpov.txt").split("\n").map(|line| {let mut line = line.splitn(2, " "); (line.next().unwrap(), line.next().unwrap())}).collect();

        let mut tic = TicTacToe::default();

        moves.into_iter().for_each(|(alg, res_fen)| test_valid_move(alg, res_fen, &mut tic));
    }

    #[test]
    fn arg_carmined() {
        
        let moves: Vec<(&str, &str)> = include_str!("arg_carmined.txt").split("\n").map(|line| {let mut line = line.splitn(2, " "); (line.next().unwrap(), line.next().unwrap())}).collect();

        let mut tic = TicTacToe::default();

        moves.into_iter().for_each(|(alg, res_fen)| test_valid_move(alg, res_fen, &mut tic));
    }

    #[test]
    fn drawn() {
        let moves: [(&str, &str); 8] = [
            ("Ng1f3", "rnbqkbnr/pppppppp/8/8/8/5N2/PPPPPPPP/RNBQKB1R b KQkq - 1 1"),
            ("Ng8f6", "rnbqkb1r/pppppppp/5n2/8/8/5N2/PPPPPPPP/RNBQKB1R w KQkq - 2 2"),
            ("Nf3g1", "rnbqkb1r/pppppppp/5n2/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 3 2"),
            ("Nf6g8", "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 4 3"),
            ("Ng1f3", "rnbqkbnr/pppppppp/8/8/8/5N2/PPPPPPPP/RNBQKB1R b KQkq - 5 3"),
            ("Ng8f6", "rnbqkb1r/pppppppp/5n2/8/8/5N2/PPPPPPPP/RNBQKB1R w KQkq - 6 4"),
            ("Nf3g1", "rnbqkb1r/pppppppp/5n2/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 7 4"),
            ("Nf6g8", "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 8 5"),
        ];

        let mut tic = TicTacToe::default();

        moves.into_iter().for_each(|(alg, res_fen)| test_valid_move(alg, res_fen, &mut tic));

        assert_eq!(tic.chesses[COORDS.0][COORDS.1].end, EndResult::Draw(true));

        let res = tic.make_move(COORDS, "e2e4");

        assert!(res.is_err());

    }

    #[test]
    fn rook_bug() {
        let mut tic = TicTacToe::default();
        tic.chesses[0][0] = ChessBoard::parse_fen("2bqkb1r/ppp2ppp/2n1pn2/3p4/2PP1B2/2R1P3/rP3ppp/1N1QKBN1 w KQkq - 0 0").unwrap();

        let chers_move = "Qd1a1";

        // println!("{:?}", tic.make_move((0, 0), chers_move));

        assert!(tic.validate_move((0, 0), chers_move).is_err() || !tic.validate_move((0, 0), chers_move).unwrap());

    }

}