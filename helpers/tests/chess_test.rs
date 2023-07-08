//TODO: invalid moves

pub mod tests {
  use helpers::{
    chess::{ChessBoard, PieceName},
    chesstactoe::{chess::EndResult, Color},
    tictactoe::TicTacToe,
    Coordinates, FenError,
  };

  const COORDS: Coordinates = Coordinates::new((0, 0));

  fn test_valid_move(alg: &str, res_fen: &str, tic: &mut TicTacToe) {
    println!("{alg}");

    assert!(tic
      .validate_move(COORDS, alg)
      .is_ok_and(|is_valid| is_valid));

    tic.make_move(COORDS, alg).unwrap();

    assert_eq!(
      tic.get_board(COORDS).unwrap().to_fen(tic.next).unwrap(),
      res_fen
    );
  }
  #[test]
  fn kasparov_karpov() {
    let moves: Vec<(&str, &str)> = include_str!("Kasparov_Karpov.txt")
      .split('\n')
      .map(|line| {
        let mut line = line.splitn(2, ' ');
        (line.next().unwrap(), line.next().unwrap())
      })
      .collect();

    let mut tic = TicTacToe::default();

    moves
      .into_iter()
      .for_each(|(alg, res_fen)| test_valid_move(alg, res_fen, &mut tic));
  }

  #[test]
  fn arg_carmined() {
    let moves: Vec<(&str, &str)> = include_str!("arg_carmined.txt")
      .split('\n')
      .map(|line| {
        let mut line = line.splitn(2, ' ');
        (line.next().unwrap(), line.next().unwrap())
      })
      .collect();

    let mut tic = TicTacToe::default();

    moves
      .into_iter()
      .for_each(|(alg, res_fen)| test_valid_move(alg, res_fen, &mut tic));
  }

  #[test]
  fn drawn() {
    let moves: [(&str, &str); 8] = [
      (
        "Ng1f3",
        "rnbqkbnr/pppppppp/8/8/8/5N2/PPPPPPPP/RNBQKB1R b KQkq - 1 1",
      ),
      (
        "Ng8f6",
        "rnbqkb1r/pppppppp/5n2/8/8/5N2/PPPPPPPP/RNBQKB1R w KQkq - 2 2",
      ),
      (
        "Nf3g1",
        "rnbqkb1r/pppppppp/5n2/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 3 2",
      ),
      (
        "Nf6g8",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 4 3",
      ),
      (
        "Ng1f3",
        "rnbqkbnr/pppppppp/8/8/8/5N2/PPPPPPPP/RNBQKB1R b KQkq - 5 3",
      ),
      (
        "Ng8f6",
        "rnbqkb1r/pppppppp/5n2/8/8/5N2/PPPPPPPP/RNBQKB1R w KQkq - 6 4",
      ),
      (
        "Nf3g1",
        "rnbqkb1r/pppppppp/5n2/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 7 4",
      ),
      (
        "Nf6g8",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 8 5",
      ),
    ];

    let mut tic = TicTacToe::default();

    moves
      .into_iter()
      .for_each(|(alg, res_fen)| test_valid_move(alg, res_fen, &mut tic));

    assert_eq!(
      tic.chesses[COORDS.row][COORDS.col].end,
      EndResult::Draw(true)
    );

    let res = tic.make_move(COORDS, "e2e4");

    assert!(res.is_err());
  }

  #[test]
  fn rook_bug() {
    let mut tic = TicTacToe::default();
    tic.chesses[0][0] =
      ChessBoard::parse_fen("2bqkb1r/ppp2ppp/2n1pn2/3p4/2PP1B2/2R1P3/rP3ppp/1N1QKBN1 w KQkq - 0 0")
        .unwrap();

    let chers_move = "Qd1a1";

    // println!("{:?}", tic.make_move((0, 0), chers_move));

    assert!(
      tic.validate_move(COORDS, chers_move).is_err()
        || !tic.validate_move(COORDS, chers_move).unwrap()
    );
  }

  #[test]
  fn pawn_bug() {
    let mut tic = TicTacToe::default();
    tic.chesses[0][0] =
      ChessBoard::parse_fen("rnbqkbnr/pppppppp/8/8/8/Q7/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();

    let chers_move = "a2a4";

    // println!("{:?}", tic.make_move((0, 0), chers_move));

    assert!(
      tic.validate_move(COORDS, chers_move).is_err()
        || !tic.validate_move(COORDS, chers_move).unwrap()
    );
  }

  #[test]
  fn test_en_passant() {
    let mut tic = TicTacToe::default();

    let moves = ["d2d4", "e7e5", "d4xe5", "d7d5"];

    moves.iter().for_each(|alg| {
      println!("{alg}");
      tic.make_move(COORDS, alg).unwrap()
    });

    assert_eq!(
      "rnbqkbnr/ppp2ppp/8/3pP3/8/8/PPP1PPPP/RNBQKBNR w KQkq d6 0 3",
      tic.get_board(COORDS).unwrap().to_fen(Color::White).unwrap()
    );

    let take_passant = "e5xd6";

    tic.make_move(COORDS, take_passant).unwrap();

    assert_eq!(
      "rnbqkbnr/ppp2ppp/3P4/8/8/8/PPP1PPPP/RNBQKBNR b KQkq - 0 3",
      tic.get_board(COORDS).unwrap().to_fen(Color::Black).unwrap()
    );
  }

  #[test]
  fn test_fen_conversion() {
    const FEN1: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 0";

    test_fen(FEN1);

    const FEN2: &str = "Kn5Q/1P1pPN2/8/2P5/p3N3/1P2P3/1pk2p1n/8 w - - 0 1";

    test_fen(FEN2);

    const FEN3: &str = "1K3BQ1/p1P5/5p2/pp3Pp1/6P1/4b3/Pk6/r3n3 w - - 0 1";

    test_fen(FEN3);

    let random_board = ChessBoard::random();

    let fen4: &str = &random_board.to_fen(Color::White).unwrap();

    let parsed_board = ChessBoard::try_from(fen4).unwrap();

    assert_eq!(parsed_board, random_board);
  }

  fn test_fen(fen: &str) {
    assert_eq!(
      ChessBoard::parse_fen(fen)
        .unwrap()
        .to_fen(Color::White)
        .unwrap(),
      fen
    );
  }

  #[test]
  fn test_long_castles() {
    let mut tic = TicTacToe::default();

    let moves = [
      ("Nb1c3", "rnbqkbnr/pppppppp/8/8/8/2N5/PPPPPPPP/R1BQKBNR b KQkq - 1 1"),
      ("Nb8c6", "r1bqkbnr/pppppppp/2n5/8/8/2N5/PPPPPPPP/R1BQKBNR w KQkq - 2 2"),
      ("d2d3", "r1bqkbnr/pppppppp/2n5/8/8/2NP4/PPP1PPPP/R1BQKBNR b KQkq - 0 2"),
      ("d7d6", "r1bqkbnr/ppp1pppp/2np4/8/8/2NP4/PPP1PPPP/R1BQKBNR w KQkq - 0 3"),
      ("Bc1e3", "r1bqkbnr/ppp1pppp/2np4/8/8/2NPB3/PPP1PPPP/R2QKBNR b KQkq - 1 3"),
      ("Bc8e6", "r2qkbnr/ppp1pppp/2npb3/8/8/2NPB3/PPP1PPPP/R2QKBNR w KQkq - 2 4"),
      ("Qd1d2", "r2qkbnr/ppp1pppp/2npb3/8/8/2NPB3/PPPQPPPP/R3KBNR b KQkq - 3 4"),
      ("Qd8d7", "r3kbnr/pppqpppp/2npb3/8/8/2NPB3/PPPQPPPP/R3KBNR w KQkq - 4 5"),
      ("O-O-O", "r3kbnr/pppqpppp/2npb3/8/8/2NPB3/PPPQPPPP/2KR1BNR b kq - 5 5"),
      ("O-O-O", "2kr1bnr/pppqpppp/2npb3/8/8/2NPB3/PPPQPPPP/2KR1BNR w - - 6 6")
    ];

    moves.iter().for_each(|(alg, fen)| {
      test_valid_move(alg, &fen, &mut tic)
    });

    let invalid1  = [
      "Nb1c3",
      "Nb8c6",
      "d2d3",
      "d7d6",
      "Bc1e3",
      "Bc8e6",
      // "Qd1d2",
      // "Qd8d7",
    ];

    let the_move = "O-O-O";

    tic = TicTacToe::default();

    invalid1.iter().for_each(|alg| {
      println!("{alg}");
      tic.make_move(COORDS, alg).unwrap();
    });

    assert!(tic.make_move(COORDS, the_move).is_err());

    let invalid2  = [
      "Nb1c3",
      "Nb8c6",
      "d2d3",
      "d7d6",
      "Bc1e3",
      "Bc8e6",
      "Qd1d2",
      // "Qd8d7",
    ];

    tic = TicTacToe::default();

    invalid2.iter().for_each(|alg| {
      tic.make_move(COORDS, alg).unwrap();
    });

    assert!(tic.make_move(COORDS, the_move).is_err());

    let invalid3  = [
      "Nb1c3",
      "Nb8c6",
      "d2d3",
      "d7d6",
      // "Bc1e3",
      // "Bc8e6",
      "Qd1d2",
      "Qd8d7",
    ];

    tic = TicTacToe::default();

    invalid3.iter().for_each(|alg| {
      tic.make_move(COORDS, alg).unwrap();
    });

    assert!(tic.make_move(COORDS, the_move).is_err());

    let invalid4 = [
      "Nb1c3",
      "Nb8c6",
      "d2d3",
      "d7d6",
      "Bc1e3",
      // "Bc8e6",
      "Qd8d7",
      "Qd1d2",
    ];

    tic = TicTacToe::default();

    invalid4.iter().for_each(|alg| {
      tic.make_move(COORDS, alg).unwrap();
    });

    assert!(tic.make_move(COORDS, the_move).is_err());


    let invalid5 = [
      // "Nb1c3",
      // "Nb8c6",
      "d2d3",
      "d7d6",
      "Bc1e3",
      "Bc8e6",
      "Qd1d2",
      "Qd8d7",
    ];

    tic = TicTacToe::default();

    invalid5.iter().for_each(|alg| {
      tic.make_move(COORDS, alg).unwrap();
    });

    assert!(tic.make_move(COORDS, the_move).is_err());

    let invalid6 = [
      "Nb1c3",
      // "Nb8c6",
      "d7d6",
      "d2d3",
      "Bc8e6",
      "Bc1e3",
      "Qd8d7",
      "Qd1d2",
    ];

    tic = TicTacToe::default();

    invalid6.iter().for_each(|alg| {
      tic.make_move(COORDS, alg).unwrap();
    });

    assert!(tic.make_move(COORDS, the_move).is_err());
  }

  #[test]
  #[should_panic]
  fn test_invalid_move_in_valid_move() {
    let mut tic = TicTacToe::default();

    test_valid_move("O-O", "Bármi", &mut tic)
  }

  #[test]
  fn bruh_test() {
    let mut tic = TicTacToe::default();

    let res = tic.make_move(COORDS, "d2d5").err().unwrap();

    println!("{res}");

    let color = Color::Black;

    assert_eq!(String::from("Black"), format!("{color}"));

    let color = Color::White;

    assert_eq!(String::from("White"), format!("{color}"));

    const INVALID_FEN: &str = "invalid fen";

    let fen_error = ChessBoard::try_from(INVALID_FEN).err().unwrap();

    assert_eq!(String::from("InvalidFormat"), format!("{fen_error}"));

    assert_eq!(Coordinates { row: 0, col: 0 }, Coordinates::new((0, 0)));

    assert_eq!(String::from("Rook"), format!("{}", PieceName::ROOK));

    assert_eq!(String::from("Knight"), format!("{}", PieceName::KNIGHT));

    assert_eq!(String::from("Bishop"), format!("{}", PieceName::BISHOP));

    assert_eq!(String::from("Queen"), format!("{}", PieceName::QUEEN));

    assert_eq!(String::from("King"), format!("{}", PieceName::KING));

    assert_eq!(String::from("Pawn"), format!("{}", PieceName::PAWN));

    assert_eq!(
      String::from("MoveError(InvalidMove)"),
      format!(
        "{}",
        ChessBoard::default()
          .make_move("Qb1b2", Color::White)
          .err()
          .unwrap()
      )
    );

    assert_eq!(
      FenError::InvalidFormat,
      ChessBoard::try_from("0 0 0 0 0 0").err().unwrap()
    );
    assert_eq!(
      FenError::InvalidFormat,
      ChessBoard::try_from("f/f/f/f/f/f/f/f 0 0 0 0 0")
        .err()
        .unwrap()
    );
    assert_eq!(
      FenError::InvalidFormat,
      ChessBoard::try_from("ppppppp/f/f/f/f/f/f/f 0 0 0 0 0")
        .err()
        .unwrap()
    );
    // assert_eq!(FenError::InvalidFormat, ChessBoard::try_from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 1 1"));
  }
}
