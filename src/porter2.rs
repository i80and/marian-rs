#![allow(unknown_lints, clippy)]

use std::cmp;
use smallvec::SmallVec;

struct Among {
    s: &'static str,
    substring_i: i32,
    result: i32,
}

impl Among {
    fn new(s: &'static str, substring_i: i32, result: i32) -> Self {
        Among {
            s,
            substring_i,
            result,
        }
    }
}

struct Stemmer {
    a_0: Vec<Among>,
    a_1: Vec<Among>,
    a_2: Vec<Among>,
    a_3: Vec<Among>,
    a_4: Vec<Among>,
    a_5: Vec<Among>,
    a_6: Vec<Among>,
    a_7: Vec<Among>,
    a_8: Vec<Among>,
    a_9: Vec<Among>,
    a_10: Vec<Among>,
    g_v: Vec<i32>,
    g_v_wxy: Vec<i32>,
    g_valid_li: Vec<i32>,
}

impl Stemmer {
    fn new() -> Self {
        Self {
            a_0: vec![
                Among::new("arsen", -1, -1),
                Among::new("commun", -1, -1),
                Among::new("gener", -1, -1),
            ],

            a_1: vec![
                Among::new("'", -1, 1),
                Among::new("'s'", 0, 1),
                Among::new("'s", -1, 1),
            ],

            a_2: vec![
                Among::new("ied", -1, 2),
                Among::new("s", -1, 3),
                Among::new("ies", 1, 2),
                Among::new("sses", 1, 1),
                Among::new("ss", 1, -1),
                Among::new("us", 1, -1),
            ],

            a_3: vec![
                Among::new("", -1, 3),
                Among::new("bb", 0, 2),
                Among::new("dd", 0, 2),
                Among::new("ff", 0, 2),
                Among::new("gg", 0, 2),
                Among::new("bl", 0, 1),
                Among::new("mm", 0, 2),
                Among::new("nn", 0, 2),
                Among::new("pp", 0, 2),
                Among::new("rr", 0, 2),
                Among::new("at", 0, 1),
                Among::new("tt", 0, 2),
                Among::new("iz", 0, 1),
            ],

            a_4: vec![
                Among::new("ed", -1, 2),
                Among::new("eed", 0, 1),
                Among::new("ing", -1, 2),
                Among::new("edly", -1, 2),
                Among::new("eedly", 3, 1),
                Among::new("ingly", -1, 2),
            ],

            a_5: vec![
                Among::new("anci", -1, 3),
                Among::new("enci", -1, 2),
                Among::new("ogi", -1, 13),
                Among::new("li", -1, 16),
                Among::new("bli", 3, 12),
                Among::new("abli", 4, 4),
                Among::new("alli", 3, 8),
                Among::new("fulli", 3, 14),
                Among::new("lessli", 3, 15),
                Among::new("ousli", 3, 10),
                Among::new("entli", 3, 5),
                Among::new("aliti", -1, 8),
                Among::new("biliti", -1, 12),
                Among::new("iviti", -1, 11),
                Among::new("tional", -1, 1),
                Among::new("ational", 14, 7),
                Among::new("alism", -1, 8),
                Among::new("ation", -1, 7),
                Among::new("ization", 17, 6),
                Among::new("izer", -1, 6),
                Among::new("ator", -1, 7),
                Among::new("iveness", -1, 11),
                Among::new("fulness", -1, 9),
                Among::new("ousness", -1, 10),
            ],

            a_6: vec![
                Among::new("icate", -1, 4),
                Among::new("ative", -1, 6),
                Among::new("alize", -1, 3),
                Among::new("iciti", -1, 4),
                Among::new("ical", -1, 4),
                Among::new("tional", -1, 1),
                Among::new("ational", 5, 2),
                Among::new("ful", -1, 5),
                Among::new("ness", -1, 5),
            ],

            a_7: vec![
                Among::new("ic", -1, 1),
                Among::new("ance", -1, 1),
                Among::new("ence", -1, 1),
                Among::new("able", -1, 1),
                Among::new("ible", -1, 1),
                Among::new("ate", -1, 1),
                Among::new("ive", -1, 1),
                Among::new("ize", -1, 1),
                Among::new("iti", -1, 1),
                Among::new("al", -1, 1),
                Among::new("ism", -1, 1),
                Among::new("ion", -1, 2),
                Among::new("er", -1, 1),
                Among::new("ous", -1, 1),
                Among::new("ant", -1, 1),
                Among::new("ent", -1, 1),
                Among::new("ment", 15, 1),
                Among::new("ement", 16, 1),
            ],

            a_8: vec![Among::new("e", -1, 1), Among::new("l", -1, 2)],

            a_9: vec![
                Among::new("succeed", -1, -1),
                Among::new("proceed", -1, -1),
                Among::new("exceed", -1, -1),
                Among::new("canning", -1, -1),
                Among::new("inning", -1, -1),
                Among::new("earring", -1, -1),
                Among::new("herring", -1, -1),
                Among::new("outing", -1, -1),
            ],

            a_10: vec![
                Among::new("andes", -1, -1),
                Among::new("atlas", -1, -1),
                Among::new("bias", -1, -1),
                Among::new("cosmos", -1, -1),
                Among::new("dying", -1, 3),
                Among::new("early", -1, 11),
                Among::new("gently", -1, 9),
                Among::new("howe", -1, -1),
                Among::new("idly", -1, 8),
                Among::new("importance", -1, 7),
                Among::new("important", -1, -1),
                Among::new("lying", -1, 4),
                Among::new("news", -1, -1),
                Among::new("only", -1, 12),
                Among::new("replica", -1, 6),
                Among::new("singly", -1, 13),
                Among::new("skies", -1, 2),
                Among::new("skis", -1, 1),
                Among::new("sky", -1, -1),
                Among::new("tying", -1, 5),
                Among::new("ugly", -1, 10),
            ],

            g_v: vec![17, 65, 16, 1],
            g_v_wxy: vec![1, 17, 65, 208, 1],
            g_valid_li: vec![55, 141, 2],
        }
    }
}

lazy_static! {
    static ref STEMMER: Stemmer = Stemmer::new();
}

pub struct StemmerContext {
    stemmer: &'static Stemmer,
    b_y_found: bool,
    i_p2: i32,
    i_p1: i32,

    current: SmallVec<[char; 16]>,
    cursor: i32,
    limit: i32,
    limit_backward: i32,
    bra: i32,
    ket: i32,
}

impl StemmerContext {
    pub fn new(value: &str) -> Self {
        let current: SmallVec<_> = value.chars().collect();
        let len = current.len() as i32;
        let mut ctx = Self {
            stemmer: &STEMMER,
            b_y_found: false,
            i_p2: 0,
            i_p1: 0,

            current,
            cursor: 0,
            limit: len,
            limit_backward: 0,
            bra: 0,
            ket: len,
        };

        ctx.stem();
        ctx
    }

    pub fn get(&self) -> String {
        let mut s = String::with_capacity(self.current.len());
        s.extend(self.current.iter());
        s
    }

    fn stem(&mut self) -> bool {
        // (, line 208
        // or, line 210
        let mut _lab0 = true;
        'lab0: while _lab0 {
            _lab0 = false;
            let v_1 = self.cursor;
            let mut _lab1 = true;
            'lab1: while _lab1 {
                _lab1 = false;
                // call exception1, line 210
                if !self.r_exception1() {
                    break 'lab1;
                }
                break 'lab0;
            }
            self.cursor = v_1;
            let mut _lab2 = true;
            'lab2: while _lab2 {
                _lab2 = false;
                // not, line 211
                {
                    let v_2 = self.cursor;
                    let mut _lab3 = true;
                    'lab3: while _lab3 {
                        _lab3 = false;
                        // hop, line 211
                        {
                            let c = self.cursor + 3;
                            if 0 > c || c > self.limit {
                                break 'lab3;
                            }
                            self.cursor = c;
                        }
                        break 'lab2;
                    }
                    self.cursor = v_2;
                }
                break 'lab0;
            }
            self.cursor = v_1;
            // (, line 211
            // do, line 212
            let v_3 = self.cursor;
            let mut _lab4 = true;
            'lab4: while _lab4 {
                _lab4 = false;
                // call prelude, line 212
                if !self.r_prelude() {
                    break 'lab4;
                }
            }
            self.cursor = v_3;
            // do, line 213
            let v_4 = self.cursor;
            let mut _lab5 = true;
            'lab5: while _lab5 {
                _lab5 = false;
                // call mark_regions, line 213
                if !self.r_mark_regions() {
                    break 'lab5;
                }
            }
            self.cursor = v_4;
            // backwards, line 214
            self.limit_backward = self.cursor;
            self.cursor = self.limit;
            // (, line 214
            // do, line 216
            let v_5 = self.limit - self.cursor;
            let mut _lab6 = true;
            'lab6: while _lab6 {
                _lab6 = false;
                // call step_1a, line 216
                if !self.r_step_1a() {
                    break 'lab6;
                }
            }
            self.cursor = self.limit - v_5;
            // or, line 218
            let mut _lab7 = true;
            'lab7: while _lab7 {
                _lab7 = false;
                let v_6 = self.limit - self.cursor;
                let mut _lab8 = true;
                'lab8: while _lab8 {
                    _lab8 = false;
                    // call exception2, line 218
                    if !self.r_exception2() {
                        break 'lab8;
                    }
                    break 'lab7;
                }
                self.cursor = self.limit - v_6;
                // (, line 218
                // do, line 220
                let v_7 = self.limit - self.cursor;
                let mut _lab9 = true;
                'lab9: while _lab9 {
                    _lab9 = false;
                    // call step_1b, line 220
                    if !self.r_step_1b() {
                        break 'lab9;
                    }
                }
                self.cursor = self.limit - v_7;
                // do, line 221
                let v_8 = self.limit - self.cursor;
                let mut _lab10 = true;
                'lab10: while _lab10 {
                    _lab10 = false;
                    // call step_1c, line 221
                    if !self.r_step_1c() {
                        break 'lab10;
                    }
                }
                self.cursor = self.limit - v_8;
                // do, line 223
                let v_9 = self.limit - self.cursor;
                let mut _lab11 = true;
                'lab11: while _lab11 {
                    _lab11 = false;
                    // call step_2, line 223
                    if !self.r_step_2() {
                        break 'lab11;
                    }
                }
                self.cursor = self.limit - v_9;
                // do, line 224
                let v_10 = self.limit - self.cursor;
                let mut _lab12 = true;
                'lab12: while _lab12 {
                    _lab12 = false;
                    // call step_3, line 224
                    if !self.r_step_3() {
                        break 'lab12;
                    }
                }
                self.cursor = self.limit - v_10;
                // do, line 225
                let v_11 = self.limit - self.cursor;
                let mut _lab13 = true;
                'lab13: while _lab13 {
                    _lab13 = false;
                    // call step_4, line 225
                    if !self.r_step_4() {
                        break 'lab13;
                    }
                }
                self.cursor = self.limit - v_11;
                // do, line 227
                let v_12 = self.limit - self.cursor;
                let mut _lab14 = true;
                'lab14: while _lab14 {
                    _lab14 = false;
                    // call step_5, line 227
                    if !self.r_step_5() {
                        break 'lab14;
                    }
                }
                self.cursor = self.limit - v_12;
            }
            self.cursor = self.limit_backward;
            // do, line 230
            let v_13 = self.cursor;
            let mut _lab15 = true;
            'lab15: while _lab15 {
                _lab15 = false;
                // call postlude, line 230
                if !self.r_postlude() {
                    break 'lab15;
                }
            }
            self.cursor = v_13;
        }

        true
    }

    fn r_mark_regions(&mut self) -> bool {
        // (, line 32
        self.i_p1 = self.limit as i32;
        self.i_p2 = self.limit as i32;
        // do, line 35
        let v_1 = self.cursor;
        let mut _lab0 = true;
        'lab0: while _lab0 {
            _lab0 = false;
            // (, line 35
            // or, line 41
            let mut _lab1 = true;
            'lab1: while _lab1 {
                _lab1 = false;
                let v_2 = self.cursor;
                let mut _lab2 = true;
                'lab2: while _lab2 {
                    _lab2 = false;
                    // among, line 36
                    if self.find_among(&self.stemmer.a_0) == 0 {
                        break 'lab2;
                    }
                    break 'lab1;
                }
                self.cursor = v_2;
                // (, line 41
                // gopast, line 41
                'golab3: loop {
                    let mut _lab4 = true;
                    'lab4: while _lab4 {
                        _lab4 = false;
                        if !self.in_grouping(&self.stemmer.g_v, 97, 121) {
                            break 'lab4;
                        }
                        break 'golab3;
                    }
                    if self.cursor >= self.limit {
                        break 'lab0;
                    }
                    self.cursor += 1;
                }
                // gopast, line 41
                'golab5: loop {
                    let mut _lab6 = true;
                    'lab6: while _lab6 {
                        _lab6 = false;
                        if !self.out_grouping(&self.stemmer.g_v, 97, 121) {
                            break 'lab6;
                        }
                        break 'golab5;
                    }
                    if self.cursor >= self.limit {
                        break 'lab0;
                    }
                    self.cursor += 1;
                }
            }
            // setmark p1, line 42
            self.i_p1 = self.cursor as i32;
            // gopast, line 43
            'golab7: loop {
                let mut _lab8 = true;
                'lab8: while _lab8 {
                    _lab8 = false;
                    if !self.in_grouping(&self.stemmer.g_v, 97, 121) {
                        break 'lab8;
                    }
                    break 'golab7;
                }
                if self.cursor >= self.limit {
                    break 'lab0;
                }
                self.cursor += 1;
            }
            // gopast, line 43
            'golab9: loop {
                let mut _lab10 = true;
                'lab10: while _lab10 {
                    _lab10 = false;
                    if !self.out_grouping(&self.stemmer.g_v, 97, 121) {
                        break 'lab10;
                    }
                    break 'golab9;
                }
                if self.cursor >= self.limit {
                    break 'lab0;
                }
                self.cursor += 1;
            }
            // setmark p2, line 43
            self.i_p2 = self.cursor as i32;
        }
        self.cursor = v_1;
        true
    }

    fn r_shortv(&mut self) -> bool {
        // (, line 49
        // or, line 51
        let mut _lab0 = true;
        'lab0: while _lab0 {
            _lab0 = false;
            let v_1 = self.limit - self.cursor;
            let mut _lab1 = true;
            'lab1: while _lab1 {
                _lab1 = false;
                // (, line 50
                if !self.out_grouping_b(&self.stemmer.g_v_wxy, 89, 121) {
                    break 'lab1;
                }
                if !self.in_grouping_b(&self.stemmer.g_v, 97, 121) {
                    break 'lab1;
                }
                if !self.out_grouping_b(&self.stemmer.g_v, 97, 121) {
                    break 'lab1;
                }
                break 'lab0;
            }
            self.cursor = self.limit - v_1;
            // (, line 52
            if !self.out_grouping_b(&self.stemmer.g_v, 97, 121) {
                return false;
            }
            if !self.in_grouping_b(&self.stemmer.g_v, 97, 121) {
                return false;
            }
            // atlimit, line 52
            if self.cursor > self.limit_backward {
                return false;
            }
        }
        true
    }

    fn r_r1(&self) -> bool {
        if !(self.i_p1 <= self.cursor as i32) {
            return false;
        }

        true
    }

    fn r_r2(&self) -> bool {
        if !(self.i_p2 <= self.cursor as i32) {
            return false;
        }

        true
    }

    fn r_prelude(&mut self) -> bool {
        // (, line 25
        // unset Y_found, line 26
        self.b_y_found = false;
        // do, line 27
        let v_1 = self.cursor;
        let mut _lab0 = true;
        while _lab0 {
            _lab0 = false;
            // (, line 27
            // [, line 27
            self.bra = self.cursor;
            // literal, line 27
            if !(self.eq_s(&['\''])) {
                break;
            }
            // ], line 27
            self.ket = self.cursor;
            // delete, line 27
            if !self.slice_del() {
                return false;
            }
        }
        self.cursor = v_1;
        // do, line 28
        let v_2 = self.cursor;
        let mut _lab1 = true;
        while _lab1 {
            _lab1 = false;
            // (, line 28
            // [, line 28
            self.bra = self.cursor;
            // literal, line 28
            if !(self.eq_s(&['y'])) {
                break;
            }
            // ], line 28
            self.ket = self.cursor;
            // <-, line 28
            if !self.slice_from(&['Y']) {
                return false;
            }
            // set Y_found, line 28
            self.b_y_found = true;
        }
        self.cursor = v_2;
        // do, line 29
        let v_3 = self.cursor;
        let mut _lab2 = true;
        while _lab2 {
            _lab2 = false;
            // repeat, line 29
            'replab3: loop {
                let v_4 = self.cursor;
                let mut _lab4 = true;
                'lab4: while _lab4 {
                    _lab4 = false;
                    // (, line 29
                    // goto, line 29
                    'golab5: loop {
                        let v_5 = self.cursor;
                        let mut _lab6 = true;
                        'lab6: while _lab6 {
                            _lab6 = false;
                            // (, line 29
                            if !(self.in_grouping(&self.stemmer.g_v, 97, 121)) {
                                break 'lab6;
                            }
                            // [, line 29
                            self.bra = self.cursor;
                            // literal, line 29
                            if !self.eq_s(&['y']) {
                                break 'lab6;
                            }
                            // ], line 29
                            self.ket = self.cursor;
                            self.cursor = v_5;
                            break 'golab5;
                        }
                        self.cursor = v_5;
                        if self.cursor >= self.limit {
                            break 'lab4;
                        }
                        self.cursor += 1;
                    }
                    // <-, line 29
                    if !self.slice_from(&['Y']) {
                        return false;
                    }
                    // set Y_found, line 29
                    self.b_y_found = true;
                    continue 'replab3;
                }
                self.cursor = v_4;
                break 'replab3;
            }
        }
        self.cursor = v_3;
        true
    }

    fn r_step_1a(&mut self) -> bool {
        // (, line 58
        // try, line 59
        let v_1 = self.limit - self.cursor;
        let mut _lab0 = true;
        'lab0: while _lab0 {
            _lab0 = false;
            // (, line 59
            // [, line 60
            self.ket = self.cursor;
            // substring, line 60
            let among_var = self.find_among_b(&self.stemmer.a_1);
            if among_var == 0 {
                self.cursor = self.limit - v_1;
                break 'lab0;
            }
            // ], line 60
            self.bra = self.cursor;
            match among_var {
                0 => {
                    self.cursor = self.limit - v_1;
                    break 'lab0;
                }
                1 => {
                    // (, line 62
                    // delete, line 62
                    if !self.slice_del() {
                        return false;
                    }
                }
                _ => unreachable!(),
            }
        }
        // [, line 65
        self.ket = self.cursor;
        // substring, line 65
        let among_var = self.find_among_b(&self.stemmer.a_2);
        if among_var == 0 {
            return false;
        }
        // ], line 65
        self.bra = self.cursor;
        match among_var {
            0 => return false,
            1 => {
                // (, line 66
                // <-, line 66
                if !self.slice_from(&['s', 's']) {
                    return false;
                }
            }
            2 => {
                // (, line 68
                // or, line 68
                let mut _lab1 = true;
                'lab1: while _lab1 {
                    _lab1 = false;
                    let v_2 = self.limit - self.cursor;
                    let mut _lab2 = true;
                    'lab2: while _lab2 {
                        _lab2 = false;
                        // (, line 68
                        // hop, line 68
                        {
                            let c = self.cursor - 2;
                            if self.limit_backward > c || c > self.limit {
                                break 'lab2;
                            }
                            self.cursor = c;
                        }
                        // <-, line 68
                        if !self.slice_from(&['i']) {
                            return false;
                        }
                        break 'lab1;
                    }
                    self.cursor = self.limit - v_2;
                    // <-, line 68
                    if !self.slice_from(&['i', 'e']) {
                        return false;
                    }
                }
            }
            3 => {
                // (, line 69
                // next, line 69
                if self.cursor <= self.limit_backward {
                    return false;
                }
                self.cursor -= 1;
                // gopast, line 69
                'golab3: loop {
                    let mut _lab4 = true;
                    'lab4: while _lab4 {
                        _lab4 = false;
                        if !self.in_grouping_b(&self.stemmer.g_v, 97, 121) {
                            break 'lab4;
                        }
                        break 'golab3;
                    }
                    if self.cursor <= self.limit_backward {
                        return false;
                    }
                    self.cursor -= 1;
                }
                // delete, line 69
                if !self.slice_del() {
                    return false;
                }
            }
            _ => (),
        }
        true
    }

    fn r_step_1b(&mut self) -> bool {
        // (, line 74
        // [, line 75
        self.ket = self.cursor;
        // substring, line 75
        let among_var = self.find_among_b(&self.stemmer.a_4);
        if among_var == 0 {
            return false;
        }
        // ], line 75
        self.bra = self.cursor;
        match among_var {
            0 => return false,
            1 => {
                // (, line 77
                // call R1, line 77
                if !self.r_r1() {
                    return false;
                }
                // <-, line 77
                if !self.slice_from(&['e', 'e']) {
                    return false;
                }
            }
            2 => {
                // (, line 79
                // test, line 80
                let v_1 = self.limit - self.cursor;
                // gopast, line 80
                'golab0: loop {
                    let mut _lab1 = true;
                    'lab1: while _lab1 {
                        _lab1 = false;
                        if !self.in_grouping_b(&self.stemmer.g_v, 97, 121) {
                            break 'lab1;
                        }
                        break 'golab0;
                    }
                    if self.cursor <= self.limit_backward {
                        return false;
                    }
                    self.cursor -= 1;
                }
                self.cursor = self.limit - v_1;
                // delete, line 80
                if !self.slice_del() {
                    return false;
                }
                // test, line 81
                let v_3 = self.limit - self.cursor;
                // substring, line 81
                let among_var = self.find_among_b(&self.stemmer.a_3);
                if among_var == 0 {
                    return false;
                }
                self.cursor = self.limit - v_3;
                match among_var {
                    0 => return false,
                    1 => {
                        // (, line 83
                        // <+, line 83
                        {
                            let c = self.cursor;
                            self.insert(c, c, &['e']);
                            self.cursor = c;
                        }
                    }
                    2 => {
                        // (, line 86
                        // [, line 86
                        self.ket = self.cursor;
                        // next, line 86
                        if self.cursor <= self.limit_backward {
                            return false;
                        }
                        self.cursor -= 1;
                        // ], line 86
                        self.bra = self.cursor;
                        // delete, line 86
                        if !self.slice_del() {
                            return false;
                        }
                    }
                    3 => {
                        // (, line 87
                        // atmark, line 87
                        if self.cursor as i32 != self.i_p1 {
                            return false;
                        }
                        // test, line 87
                        let v_4 = self.limit - self.cursor;
                        // call shortv, line 87
                        if !self.r_shortv() {
                            return false;
                        }
                        self.cursor = self.limit - v_4;
                        // <+, line 87
                        {
                            let c = self.cursor;
                            self.insert(c, c, &['e']);
                            self.cursor = c;
                        }
                    }
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
        true
    }

    fn r_step_1c(&mut self) -> bool {
        // (, line 93
        // [, line 94
        self.ket = self.cursor;
        // or, line 94
        let mut _lab0 = true;
        'lab0: while _lab0 {
            _lab0 = false;
            let v_1 = self.limit - self.cursor;
            let mut _lab1 = true;
            'lab1: while _lab1 {
                _lab1 = false;
                // literal, line 94
                if !self.eq_s_b(&['y']) {
                    break 'lab1;
                }
                break 'lab0;
            }
            self.cursor = self.limit - v_1;
            // literal, line 94
            if !self.eq_s_b(&['Y']) {
                return false;
            }
        }
        // ], line 94
        self.bra = self.cursor;
        if !self.out_grouping_b(&self.stemmer.g_v, 97, 121) {
            return false;
        }
        // not, line 95
        {
            let v_2 = self.limit - self.cursor;
            let mut _lab2 = true;
            'lab2: while _lab2 {
                _lab2 = false;
                // atlimit, line 95
                if self.cursor > self.limit_backward {
                    break 'lab2;
                }
                return false;
            }
            self.cursor = self.limit - v_2;
        }
        // <-, line 96
        if !self.slice_from(&['i']) {
            return false;
        }
        true
    }

    fn r_step_2(&mut self) -> bool {
        // (, line 99
        // [, line 100
        self.ket = self.cursor;
        // substring, line 100
        let among_var = self.find_among_b(&self.stemmer.a_5);
        if among_var == 0 {
            return false;
        }
        // ], line 100
        self.bra = self.cursor;

        // call R1, line 100
        if !self.r_r1() {
            return false;
        }

        match among_var {
            0 => return false,
            1 => {
                // (, line 101
                // <-, line 101
                if !self.slice_from(&['t', 'i', 'o', 'n']) {
                    return false;
                }
            }
            2 => {
                // (, line 102
                // <-, line 102
                if !self.slice_from(&['e', 'n', 'c', 'e']) {
                    return false;
                }
            }
            3 => {
                // (, line 103
                // <-, line 103
                if !self.slice_from(&['a', 'n', 'c', 'e']) {
                    return false;
                }
            }
            4 => {
                // (, line 104
                // <-, line 104
                if !self.slice_from(&['a', 'b', 'l', 'e']) {
                    return false;
                }
            }
            5 => {
                // (, line 105
                // <-, line 105
                if !self.slice_from(&['e', 'n', 't']) {
                    return false;
                }
            }
            6 => {
                // (, line 107
                // <-, line 107
                if !self.slice_from(&['i', 'z', 'e']) {
                    return false;
                }
            }
            7 => {
                // (, line 109
                // <-, line 109
                if !self.slice_from(&['a', 't', 'e']) {
                    return false;
                }
            }
            8 => {
                // (, line 111
                // <-, line 111
                if !self.slice_from(&['a', 'l']) {
                    return false;
                }
            }
            9 => {
                // (, line 112
                // <-, line 112
                if !self.slice_from(&['f', 'u', 'l']) {
                    return false;
                }
            }
            10 => {
                // (, line 114
                // <-, line 114
                if !self.slice_from(&['o', 'u', 's']) {
                    return false;
                }
            }
            11 => {
                // (, line 116
                // <-, line 116
                if !self.slice_from(&['i', 'v', 'e']) {
                    return false;
                }
            }
            12 => {
                // (, line 118
                // <-, line 118
                if !self.slice_from(&['b', 'l', 'e']) {
                    return false;
                }
            }
            13 => {
                // (, line 119
                // literal, line 119
                if !self.eq_s_b(&['l']) {
                    return false;
                }
                // <-, line 119
                if !self.slice_from(&['o', 'g']) {
                    return false;
                }
            }
            14 => {
                // (, line 120
                // <-, line 120
                if !self.slice_from(&['f', 'u', 'l']) {
                    return false;
                }
            }
            15 => {
                // (, line 121
                // <-, line 121
                if !self.slice_from(&['l', 'e', 's', 's']) {
                    return false;
                }
            }
            16 => {
                // (, line 122
                if !self.in_grouping_b(&self.stemmer.g_valid_li, 99, 116) {
                    return false;
                }
                // delete, line 122
                if !self.slice_del() {
                    return false;
                }
            }
            _ => unreachable!(),
        }
        true
    }

    fn r_step_3(&mut self) -> bool {
        // (, line 126
        // [, line 127
        self.ket = self.cursor;
        // substring, line 127
        let among_var = self.find_among_b(&self.stemmer.a_6);
        if among_var == 0 {
            return false;
        }
        // ], line 127
        self.bra = self.cursor;
        // call R1, line 127
        if !self.r_r1() {
            return false;
        }
        match among_var {
            0 => return false,
            1 => {
                // (, line 128
                // <-, line 128
                if !self.slice_from(&['t', 'i', 'o', 'n']) {
                    return false;
                }
            }
            2 => {
                // (, line 129
                // <-, line 129
                if !self.slice_from(&['a', 't', 'e']) {
                    return false;
                }
            }
            3 => {
                // (, line 130
                // <-, line 130
                if !self.slice_from(&['a', 'l']) {
                    return false;
                }
            }
            4 => {
                // (, line 132
                // <-, line 132
                if !self.slice_from(&['i', 'c']) {
                    return false;
                }
            }
            5 => {
                // (, line 134
                // delete, line 134
                if !self.slice_del() {
                    return false;
                }
            }
            6 => {
                // (, line 136
                // call R2, line 136
                if !self.r_r2() {
                    return false;
                }
                // delete, line 136
                if !self.slice_del() {
                    return false;
                }
            }
            _ => unreachable!(),
        }
        true
    }

    fn r_step_4(&mut self) -> bool {
        // (, line 140
        // [, line 141
        self.ket = self.cursor;
        // substring, line 141
        let among_var = self.find_among_b(&self.stemmer.a_7);
        if among_var == 0 {
            return false;
        }
        // ], line 141
        self.bra = self.cursor;
        // call R2, line 141
        if !self.r_r2() {
            return false;
        }
        match among_var {
            0 => return false,
            1 => {
                // (, line 144
                // delete, line 144
                if !self.slice_del() {
                    return false;
                }
            }
            2 => {
                // (, line 145
                // or, line 145
                let mut _lab0 = true;
                'lab0: while _lab0 {
                    _lab0 = false;
                    let v_1 = self.limit - self.cursor;
                    let mut _lab1 = true;
                    'lab1: while _lab1 {
                        _lab1 = false;
                        // literal, line 145
                        if !self.eq_s_b(&['s']) {
                            break 'lab1;
                        }
                        break 'lab0;
                    }
                    self.cursor = self.limit - v_1;
                    // literal, line 145
                    if !self.eq_s_b(&['t']) {
                        return false;
                    }
                }
                // delete, line 145
                if !self.slice_del() {
                    return false;
                }
            }
            _ => unreachable!(),
        }
        true
    }

    fn r_step_5(&mut self) -> bool {
        // (, line 149
        // [, line 150
        self.ket = self.cursor;
        // substring, line 150
        let among_var = self.find_among_b(&self.stemmer.a_8);
        if among_var == 0 {
            return false;
        }
        // ], line 150
        self.bra = self.cursor;
        match among_var {
            0 => return false,
            1 => {
                // (, line 151
                // or, line 151
                let mut _lab0 = true;
                'lab0: while _lab0 {
                    _lab0 = false;
                    let v_1 = self.limit - self.cursor;
                    let mut _lab1 = true;
                    'lab1: while _lab1 {
                        _lab1 = false;
                        // call R2, line 151
                        if !self.r_r2() {
                            break 'lab1;
                        }
                        break 'lab0;
                    }
                    self.cursor = self.limit - v_1;
                    // (, line 151
                    // call R1, line 151
                    if !self.r_r1() {
                        return false;
                    }
                    // not, line 151
                    {
                        let v_2 = self.limit - self.cursor;
                        let mut _lab2 = true;
                        'lab2: while _lab2 {
                            _lab2 = false;
                            // call shortv, line 151
                            if !self.r_shortv() {
                                break 'lab2;
                            }
                            return false;
                        }
                        self.cursor = self.limit - v_2;
                    }
                }
                // delete, line 151
                if !self.slice_del() {
                    return false;
                }
            }
            2 => {
                // (, line 152
                // call R2, line 152
                if !self.r_r2() {
                    return false;
                }
                // literal, line 152
                if !self.eq_s_b(&['l']) {
                    return false;
                }
                // delete, line 152
                if !self.slice_del() {
                    return false;
                }
            }
            _ => unreachable!(),
        }
        true
    }

    fn r_exception1(&mut self) -> bool {
        // (, line 168
        // [, line 170
        self.bra = self.cursor;
        // substring, line 170
        let among_var = self.find_among(&self.stemmer.a_10);
        if among_var == 0 {
            return false;
        }
        // ], line 170
        self.ket = self.cursor;
        // atlimit, line 170
        if self.cursor < self.limit {
            return false;
        }
        match among_var {
            0 => {
                return false;
            }
            1 => {
                // (, line 174
                // <-, line 174
                if !self.slice_from(&['s', 'k', 'i']) {
                    return false;
                }
            }
            2 => {
                // (, line 175
                // <-, line 175
                if !self.slice_from(&['s', 'k', 'y']) {
                    return false;
                }
            }
            3 => {
                // (, line 176
                // <-, line 176
                if !self.slice_from(&['d', 'i', 'e']) {
                    return false;
                }
            }
            4 => {
                // (, line 177
                // <-, line 177
                if !self.slice_from(&['l', 'i', 'e']) {
                    return false;
                }
            }
            5 => {
                // (, line 178
                // <-, line 178
                if !self.slice_from(&['t', 'i', 'e']) {
                    return false;
                }
            }
            6 => {
                // (, line 179
                // <-, line 179
                if !self.slice_from(&['r', 'e', 'p', 'l', 'i', 'c']) {
                    return false;
                }
            }
            7 => {
                // (, line 180
                // <-, line 180
                if !self.slice_from(&['i', 'm', 'p', 'o', 'r', 't', 'a', 'n', 't']) {
                    return false;
                }
            }
            8 => {
                // (, line 184
                // <-, line 184
                if !self.slice_from(&['i', 'd', 'l']) {
                    return false;
                }
            }
            9 => {
                // (, line 185
                // <-, line 185
                if !self.slice_from(&['g', 'e', 'n', 't', 'l']) {
                    return false;
                }
            }
            10 => {
                // (, line 186
                // <-, line 186
                if !self.slice_from(&['u', 'g', 'l', 'i']) {
                    return false;
                }
            }
            11 => {
                // (, line 187
                // <-, line 187
                if !self.slice_from(&['e', 'a', 'r', 'l', 'i']) {
                    return false;
                }
            }
            12 => {
                // (, line 188
                // <-, line 188
                if !self.slice_from(&['o', 'n', 'l', 'i']) {
                    return false;
                }
            }
            13 => {
                // (, line 189
                // <-, line 189
                if !self.slice_from(&['s', 'i', 'n', 'g', 'l']) {
                    return false;
                }
            }
            _ => (),
        }
        true
    }

    fn r_exception2(&mut self) -> bool {
        // (, line 156
        // [, line 158
        self.ket = self.cursor;
        // substring, line 158
        if self.find_among_b(&self.stemmer.a_9) == 0 {
            return false;
        }
        // ], line 158
        self.bra = self.cursor;
        // atlimit, line 158
        if self.cursor > self.limit_backward {
            return false;
        }
        true
    }

    fn r_postlude(&mut self) -> bool {
        // (, line 206
        // Boolean test Y_found, line 206
        if !self.b_y_found {
            return false;
        }
        // repeat, line 206
        'replab0: loop {
            let v_1 = self.cursor;
            let mut _lab1 = true;
            'lab1: while _lab1 {
                _lab1 = false;
                // (, line 206
                // goto, line 206
                'golab2: loop {
                    let v_2 = self.cursor;
                    let mut _lab3 = true;
                    'lab3: while _lab3 {
                        _lab3 = false;
                        // (, line 206
                        // [, line 206
                        self.bra = self.cursor;
                        // literal, line 206
                        if !self.eq_s(&['Y']) {
                            break 'lab3;
                        }
                        // ], line 206
                        self.ket = self.cursor;
                        self.cursor = v_2;
                        break 'golab2;
                    }
                    self.cursor = v_2;
                    if self.cursor >= self.limit {
                        break 'lab1;
                    }
                    self.cursor += 1;
                }
                // <-, line 206
                if !self.slice_from(&['y']) {
                    return false;
                }
                continue 'replab0;
            }
            self.cursor = v_1;
            break 'replab0;
        }
        true
    }

    fn in_grouping(&mut self, s: &[i32], min: u32, max: u32) -> bool {
        if self.cursor >= self.limit {
            return false;
        }

        let mut ch = self.current[self.cursor as usize] as u32;
        if ch > max || ch < min {
            return false;
        }

        ch -= min;
        if s[ch as usize >> 3] as u32 & (0x1 << (ch & 0x7)) == 0 {
            return false;
        }

        self.cursor += 1;
        true
    }

    fn in_grouping_b(&mut self, s: &[i32], min: u32, max: u32) -> bool {
        if self.cursor <= self.limit_backward {
            return false;
        }
        let mut ch = self.current[self.cursor as usize - 1] as u32;
        if ch > max || ch < min {
            return false;
        }
        ch -= min;
        if s[ch as usize >> 3] & (0x1 << (ch & 0x7)) == 0 {
            return false;
        }
        self.cursor -= 1;
        true
    }

    fn out_grouping(&mut self, s: &[i32], min: u32, max: u32) -> bool {
        if self.cursor >= self.limit {
            return false;
        }
        let mut ch = self.current[self.cursor as usize] as u32;
        if ch > max || ch < min {
            self.cursor += 1;
            return true;
        }
        ch -= min;
        if s[ch as usize >> 3] & (0x1 << (ch & 0x7)) == 0 {
            self.cursor += 1;
            return true;
        }
        false
    }

    fn out_grouping_b(&mut self, s: &[i32], min: u32, max: u32) -> bool {
        if self.cursor <= self.limit_backward {
            return false;
        }
        let mut ch = self.current[self.cursor as usize - 1] as u32;
        if ch > max || ch < min {
            self.cursor -= 1;
            return true;
        }
        ch -= min;
        if (s[ch as usize >> 3] & (0x1 << (ch & 0x7))) == 0 {
            self.cursor -= 1;
            return true;
        }
        false
    }

    fn find_among(&mut self, v: &[Among]) -> i32 {
        let mut i: i32 = 0;
        let mut j: i32 = v.len() as i32 as i32;

        let c = self.cursor;
        let l = self.limit;

        let mut common_i = 0;
        let mut common_j = 0;

        let mut first_key_inspected = false;

        loop {
            let k = i + ((j - i) >> 1);
            let mut diff: i32 = 0;
            let mut common = cmp::min(common_i, common_j);
            let w = &v[k as usize];
            for i2 in common..w.s.len() as i32 {
                if c + common == l {
                    diff = -1;
                    break;
                }
                diff = self.current[(c + common) as usize] as i32
                    - w.s.chars().nth(i2 as usize).unwrap() as i32;
                if diff != 0 {
                    break;
                }
                common += 1;
            }
            if diff < 0 {
                j = k;
                common_j = common;
            } else {
                i = k;
                common_i = common;
            }
            if j - i <= 1 {
                if i > 0 {
                    break;
                } // v->s has been inspected
                if j == i {
                    break;
                } // only one item in v

                // - but now we need to go round once more to get
                // v->s inspected. This looks messy, but is actually
                // the optimal approach.

                if first_key_inspected {
                    break;
                }
                first_key_inspected = true;
            }
        }

        loop {
            let w = &v[i as usize];
            if common_i >= w.s.len() as i32 {
                self.cursor = c + w.s.len() as i32;
                return w.result;
            }
            i = w.substring_i;
            if i < 0 {
                return 0;
            }
        }
    }

    // find_among_b is for backwards processing. Same comments apply
    fn find_among_b(&mut self, v: &[Among]) -> i32 {
        let mut i = 0;
        let mut j = v.len() as i32;

        let c = self.cursor;
        let lb = self.limit_backward;

        let mut common_i = 0;
        let mut common_j = 0;

        let mut first_key_inspected = false;

        loop {
            let k = i + ((j - i) >> 1);
            let mut diff: i32 = 0;
            let mut common = cmp::min(common_i, common_j);
            let w = &v[k as usize];

            for i2 in (0..(w.s.len() as i32 - 1 - common + 1) as i32).rev() {
                if c - common == lb {
                    diff = -1;
                    break;
                }
                diff = self.current[(c - 1 - common) as usize] as i32
                    - w.s.chars().nth(i2 as usize).unwrap() as i32;
                if diff != 0 {
                    break;
                }
                common += 1;
            }
            if diff < 0 {
                j = k;
                common_j = common;
            } else {
                i = k;
                common_i = common;
            }

            if j - i <= 1 {
                if i > 0 {
                    break;
                }
                if j == i {
                    break;
                }
                if first_key_inspected {
                    break;
                }
                first_key_inspected = true;
            }
        }

        loop {
            let w = &v[i as usize];
            if common_i >= w.s.len() as i32 {
                self.cursor = c - w.s.len() as i32;
                return w.result;
            }

            i = w.substring_i;
            if i < 0 {
                return 0;
            }
        }
    }

    /* to replace chars between c_bra and c_ket in self.current by the
     * chars in s.
     */
    fn replace_s(&mut self, c_bra: i32, c_ket: i32, s: &[char]) -> i32 {
        let adjustment = s.len() as i32 - (c_ket - c_bra);


        let new_current = {
            let part1 = &self.current[0..c_bra as usize];
            let part3 = &self.current[c_ket as usize..];
            let mut new_current = SmallVec::<[char; 16]>::new();
            new_current.extend_from_slice(part1);
            new_current.extend_from_slice(s);
            new_current.extend_from_slice(part3);

            new_current
        };

        self.current = new_current;
        self.limit += adjustment;
        if self.cursor >= c_ket {
            self.cursor += adjustment;
        } else if self.cursor > c_bra {
            self.cursor = c_bra;
        }

        adjustment
    }

    fn slice_check(&self) -> bool {
        if self.bra < 0 || self.bra > self.ket || self.ket > self.limit
            || self.limit > self.current.len() as i32
        {
            return false;
        }

        true
    }

    fn slice_from(&mut self, s: &[char]) -> bool {
        if self.slice_check() {
            let bra = self.bra;
            let ket = self.ket;
            self.replace_s(bra, ket, s);
            return true;
        }

        false
    }

    fn slice_del(&mut self) -> bool {
        self.slice_from(&[])
    }

    fn insert(&mut self, c_bra: i32, c_ket: i32, s: &[char]) {
        let adjustment = self.replace_s(c_bra, c_ket, s);
        if c_bra <= self.bra {
            self.bra += adjustment;
        }
        if c_bra <= self.ket {
            self.ket += adjustment;
        }
    }

    fn eq_s_b(&mut self, s: &[char]) -> bool {
        if self.cursor - self.limit_backward < s.len() as i32 {
            return false;
        }

        if &self.current[self.cursor as usize - s.len()..self.cursor as usize] != s {
            return false;
        }

        self.cursor -= s.len() as i32;
        true
    }

    fn eq_s(&mut self, s: &[char]) -> bool {
        if self.limit - self.cursor < s.len() as i32 {
            return false;
        }

        if &self.current[self.cursor as usize..self.cursor as usize + s.len()] != s {
            return false;
        }

        self.cursor += s.len() as i32 as i32;
        true
    }
}
