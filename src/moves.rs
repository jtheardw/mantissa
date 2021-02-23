struct Move {
    repr: String,
    start: (u32, u32),
    end: (u32, u32),
    is_ep: bool,
    is_double: bool,
    is_castle: bool,
    promote_to: u8,
    new_castling_rights: (bool, bool, bool, bool),
    is_null: bool,
    is_err: bool
}

impl Default for Move {
    fn default() -> Move {
        Move {
            repr: "",
            start: (0, 0),
            end: (0, 0),
            is_ep: false,
            is_double: false,
            is_castle: false,
            promote_to: 0,
            new_castling_rights: (true, true, true, true),
            is_null: false,
            is_err: false
        }
    }
}

impl Move {
    fn get_repr(&start: (u32, u32), &end: (u32, u32)) -> String {
        let f1 = "abcdefgh".as_bytes()[start.0];
        let r1 = (start.1 + 1).to_string();
        let f2 = "abcdefgh".as_bytes()[end.0];
        let r1 = (end.1 + 1).to_string();

        return format!("{}{}{}{}", f1, r1, f2, r2);
    }

    fn pawn_move(&node: Node, &start: (u32, u32), &end: (u32, u32)) -> Move {
        Move {
            repr: Move::get_repr(&start, &end),
            start: start,
            end: end,
            is_ep: false,
            is_double: ((start.1) - (end.1)).abs() == 2,
            is_castle: false,
            promote_to: 0,
            new_castling_rights: node.cr,
            is_null: false,
            is_err: false
        }
    }

    fn sliding_move(&node: Node, &start: (u32, u32), &end: (u32, u32)) -> Move {
        Move {
            repr: Move::get_repr(&start, &end),
            start: start,
            end: end,
            is_ep: false,
            is_double: false,
            is_castle: false,
            promote_to: 0,
            new_castling_rights: node.cr,
            is_null: false,
            is_err: false
        }
    }

    fn king_move(&node: Node, &start: (u32, u32), &end: (u32, u32)) -> Move {
        Move::sliding_move(&node, &start, &end)
    }

    fn knight_move(&node: Node, &start: (u32, u32), &end: (u32, u32)) -> Move {
        Move::sliding_move(&node, &start, &end)
    }
}
