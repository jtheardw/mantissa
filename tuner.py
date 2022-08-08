import json
import math
import os
import random
import shutil
import subprocess

ENGINE_DIR = "/home/jtwright/test_mantissas/"
BOOK_DIR = "/home/jtwright/chess/books/"
MANTISSA_DIR = "/home/jtwright/chess/mantissa_tuning/"
SEARCH_PARAMS_FILE = os.path.join(MANTISSA_DIR, "src/searchparams.rs")
COMPILE_DIR = os.path.join(MANTISSA_DIR, "target/release")

PARAMS_HISTORY_FILE = "/home/jtwright/params_history.dat"

INTEGER_PARAMS = {
    "efp_margin_base",
    "efp_margin_factor",
    "fp_margin_base",
    "fp_margin_factor",
    "rfp_margin_base",
    "rfp_margin_factor",
    "afp_margin",
    "null_move_r_denom",
    "razoring_margin",
    "lmr_history_denominator",
    "history_leaf_pruning_margin",
    "countermove_pruning_factor",
    "followup_pruning_factor",
    "singular_margin_factor"
}

PARAM_MODS = {
    # initial, min, max, step
    "efp_margin_base": (1000, 0, 4000, 100),
    "efp_margin_factor": (1200, 0, 4000, 100),

    "fp_margin_base": (1000, 0, 4000, 100),
    "fp_margin_factor": (600, 0, 4000, 100),

    "rfp_margin_base": (500, 0, 4000, 100),
    "rfp_margin_factor": (1200, 0, 4000, 100),

    "afp_margin": (30000, 10000, 50000, 3000),

    "null_move_r_base": (4.0, 0.0, 8.0, 0.2),
    "null_move_r_factor": (1./6., 0.0, 0.5, 1./30),
    "null_move_r_denom": (3000, 1000, 15000, 200),

    "razoring_margin": (2500, 1000, 5000, 100),

    "lmr_base": (0.8, 0.0, 2.0, 0.1),
    "lmr_factor": (1./2.25, 1./4, 1.0, 0.07),
    "lmr_history_denominator": (8000, 2000, 16000, 250),

    "lmp_improving_base": (4.0, 0.0, 10.0, 0.3),
    "lmp_improving_factor": (1.0, 0.1, 2.0, 0.07),
    "lmp_nonimproving_base": (2.0, 0.0, 10.0, 0.3),
    "lmp_nonimproving_factor": (0.5, 0.1, 2.0, 0.07),

    "history_decay_factor": (1.0 / 512.0, 1.0 / 1024.0, 1.0 / 100.0, 0.0001),
    "history_delta_factor": (32.0, 10.0, 50.0, 1.0),

    "history_leaf_pruning_margin": (6000, 0, 30000, 500),
    "countermove_pruning_factor": (-700, -4000, 0, 100),
    "followup_pruning_factor": (-1500, -5000, 0, 175),

    "singular_margin_factor": (37, 10, 100, 5)
}

PARAMS_FILE_TEMPLATE = """
pub const EFP_MARGIN_BASE: i32 = {efp_margin_base};
pub const EFP_MARGIN_FACTOR: i32 = {efp_margin_factor};

pub const FP_MARGIN_BASE: i32 = {fp_margin_base};
pub const FP_MARGIN_FACTOR: i32 = {fp_margin_factor};

pub const RFP_MARGIN_BASE: i32 = {rfp_margin_base};
pub const RFP_MARGIN_FACTOR: i32 = {rfp_margin_factor};

pub const AFP_MARGIN: i32 = {afp_margin};

pub const NULL_MOVE_R_BASE: f64 = {null_move_r_base};
pub const NULL_MOVE_R_FACTOR: f64 = {null_move_r_factor};
pub const NULL_MOVE_R_DENOM: i32 = {null_move_r_denom};

pub const RAZORING_MARGIN: i32 = {razoring_margin};

pub const LMR_BASE: f64 = {lmr_base};
pub const LMR_FACTOR: f64 = {lmr_factor};
pub const LMR_HISTORY_DENOMINATOR: i32 = {lmr_history_denominator};

pub const LMP_IMPROVING_BASE: f64 = {lmp_improving_base};
pub const LMP_IMPROVING_FACTOR: f64 = {lmp_improving_factor};
pub const LMP_NONIMPROVING_BASE: f64 = {lmp_nonimproving_base};
pub const LMP_NONIMPROVING_FACTOR: f64 = {lmp_nonimproving_factor};

pub const HISTORY_DECAY_FACTOR: f64 = {history_decay_factor};
pub const HISTORY_DELTA_FACTOR: f64 = {history_delta_factor};

pub const HISTORY_LEAF_PRUNING_MARGIN: i32 = {history_leaf_pruning_margin};
pub const COUNTERMOVE_PRUNING_FACTOR: i32 = {countermove_pruning_factor};
pub const FOLLOWUP_PRUNING_FACTOR: i32 = {followup_pruning_factor};

pub const SINGULAR_MARGIN_FACTOR: i32 = {singular_margin_factor};
"""


NUM_STEP_GAMES = 2
NUM_VERIFICATION_GAMES = 1024

class Params(dict):
    def __add__(self, other):
        return Params(**{k: self[k] + other[k] for k in self})
        # return ParamsVector([self[i] + other[i] for i in range(len(self))])

    def __neg__(self):
        return Params(**{k: -self[k] for k in self})

    def __sub__(self, other):
        return self + (-other)

    def __mul__(self, other):
        return Params(**{k: self[k] * other for k in self})

    def __rmul__(self, other):
        return self * other

    def __div__(self, other):
        return Params(**{k: self[k] / other for k in self})

    def normalize(self):
        return Params(**{k: self[k] / PARAM_MODS[k][3] for k in self})

    def denormalize(self):
        return Params(**{k: self[k] * PARAM_MODS[k][3] for k in self})

    def invert(self):
        return Params(**{k: 1 / self[k] for k in self})

    def __str__(self):
        return "\n".join([f'{key}: {value}' for key, value in self.items()])


INITIAL_PARAMS = Params(**{k: PARAM_MODS[k][0] for k in PARAM_MODS})
PARAMS_STEP = Params(**{k: PARAM_MODS[k][3] for k in PARAM_MODS})

def new_engine_config(engine_name):
    return {
        "command": f"./{engine_name}",
        "name": engine_name,
        "options": [
            {
                "alias": "",
                "default": 64,
                "max": 65536,
                "min": 1,
                "name": "Hash",
                "type": "spin",
                "value": 256
            },
            {
                "alias": "",
                "default": 1,
                "max": 64,
                "min": 1,
                "name": "Threads",
                "type": "spin",
                "value": 1
            },
            {
                "alias": "",
                "default": 10,
                "max": 1000,
                "min": 1,
                "name": "Move Overhead",
                "type": "spin",
                "value": 10
            }
        ],
        "protocol": "uci",
        "stderrFile": "",
        "workingDirectory": "/home/jtwright/test_mantissas"
    }


def setup_configs():
    def get_cfg_idx(config, name):
        for i in range(len(config)):
            if config[i]['name'] == name:
                return i
        return None

    cutechess_config_path = "/home/jtwright/.config/cutechess/engines.json"
    config = []
    try:
        with open(cutechess_config_path, "r") as f:
            config = json.loads(f.read())
    except Exception:
        pass
    with open(cutechess_config_path, "w") as f:
        for engine_name in ["mantissa-plus", "mantissa-minus", "mantissa-inter", "mantissa-reference"]:
            idx = get_cfg_idx(config, engine_name)
            if idx is not None:
                config.pop(idx)

            config.append(new_engine_config(engine_name))
        f.write(json.dumps(config))


def generate_base_delta():
    delta_dict = {}
    for k in PARAM_MODS:
        sign = 1 if random.randint(0, 1) else -1
        delta_dict[k] = PARAM_MODS[k][3] * sign

    return Params(**delta_dict)


def normalize_params(params):
    new_params = Params()
    for k in params:
        lo, hi = PARAM_MODS[k][1:3]
        param = params[k]
        if k in INTEGER_PARAMS and param % 1:
            # Stochastic rounding.
            param = int(math.floor(param + random.uniform(0, 1)))
        new_params[k] = min(hi, max(lo, param))
    return new_params


def run_step_games():
    cmd = ["cutechess-cli", "-tournament", "gauntlet", "-concurrency", "48", "-engine", "conf=mantissa-plus", "tc=10+0.1", "-engine", "conf=mantissa-minus", "tc=10+0.1", "-ratinginterval", "1", "-recover", "-event", "TISSA_TUNING", "-resultformat", "per-color", "-each", "book=/home/jtwright/chess/books/gm2001.bin", "bookdepth=10", "proto=uci", "option.Hash=32", "option.Threads=1", "-games", str(NUM_STEP_GAMES)]
    result = subprocess.run(cmd, capture_output=True)
    log = result.stdout.decode('utf-8')
    idx = log.rfind("Score of ")
    tail = log[idx:]
    start = tail.find(":")
    end = tail.find("[")

    score_section = tail[start+1:end].strip()
    wins, draws, losses = [int(n.strip()) for n in score_section.split(" - ")]

    return wins - losses


def _run_verification_games():
    cmd = ["cutechess-cli", "-tournament", "gauntlet", "-repeat", "-concurrency", "48", "-engine", "conf=mantissa-plus", "tc='10+0.1'", "-engine", "conf=mantissa-minus", "tc='30+0.3'", "-ratinginterval", "1", "-recover", "-event", "TISSA_TUNING", "-resultformat", "per-color", "-each", "book=/home/jtwright/books/gm2001.bin", "bookdepth=10", "proto=uci", "option.Hash=256", "option.Threads=1", "-games", str(NUM_VERIFICATION_GAMES)]
    result = subprocess.run(cmd, capture_output=True)
    log = result.stdout
    idx = log.rfind("Elo difference: ")
    tail = log[idx:]
    start = tail.find(":")
    end = tail.find("+/-")
    elo = float(tail[start+2:end-1])

    return elo


def compile_engine(params):
    with open(SEARCH_PARAMS_FILE, "w") as f:
        f.write(PARAMS_FILE_TEMPLATE.format(**params))
    os.chdir(MANTISSA_DIR)
    result = subprocess.run(os.path.join(MANTISSA_DIR, "build"), stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    assert result.returncode == 0


def setup_engines_for_step(params, delta):
    # make engine files, compile, and copy them to correct places
    params_plus = normalize_params(params + delta)
    compile_engine(params_plus)
    shutil.copyfile(os.path.join(COMPILE_DIR, 'mantissa'), os.path.join(ENGINE_DIR, f'mantissa-plus'))

    params_minus = normalize_params(params - delta)
    compile_engine(params_minus)
    shutil.copyfile(os.path.join(COMPILE_DIR, 'mantissa'), os.path.join(ENGINE_DIR, f'mantissa-minus'))


def verify_engine_strength(params, n):
    compile_engine(params)
    # for later testing if need be
    shutil.copyfile(os.path.join(COMPILE_DIR, 'mantissa'), os.path.join(ENGINE_DIR, f'mantissa-step-{n}'))

    # for immediate testing
    shutil.copyfile(os.path.join(COMPILE_DIR, 'mantissa'), os.path.join(ENGINE_DIR, f'mantissa-inter'))

    elo = run_verification_games()
    print(f"""
    VERIFICATION STEP {n}:
    ESTIMATED ELO DIFFERENCE FROM REFERENCE: {elo}
    """)


def write_params_history(params_history):
    with open(PARAMS_HISTORY_FILE, "w") as f:
        f.write('#')
        f.write('\t'.join(key for key in params_history[0]))
        f.write('\n')
        for step, params in enumerate(params_history):
            f.write('\t'.join(str(param) for key, param in params_history[step].items()))
            f.write('\n')


def tune(num_games, final_r=0.002, final_c=1.0, alpha=0.6, gamma=0.1, initial_params=INITIAL_PARAMS):
    params = Params(**initial_params.copy())
    param_history = [params]

    c0 = final_c * (1 + num_games) ** gamma
    a0 = final_r * (final_c ** 2) * (1 + num_games) ** alpha

    def c(n):
        return c0 / (1 + n) ** gamma

    def a(n):
        return a0 / (1 + n) ** alpha

    def calculate_gradient(score, delta, n):
        return (a(n) * score * delta.normalize().invert()).denormalize()
        # return Params({k: a(n) * score * delta[k] / PARAM_MODS[k][3]).invert() for k in delta})
        # return r(n) * score * delta

    setup_configs()

    for n in range(num_games):
        print(f"""
=============================
STEP {n}

PARAMS:
{str(params)}

        """)

        delta = c(n) * generate_base_delta()
        print(f"""
DELTA:
{str(delta)}

        """)

        setup_engines_for_step(params, delta)
        score = run_step_games()
        print(f"""
SCORE (wins - loss):
{score}

        """)

        gradient = calculate_gradient(score, delta, n)
        print(f"""
GRADIENT:
{gradient}


        """)

        params = params + gradient
        param_history.append(params)
        write_params_history(param_history)

if __name__ == "__main__":
    tune(num_games=10000)
