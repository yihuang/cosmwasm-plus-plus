import json

from .utils import Command

wasmd = Command("wasmd")
default_args = dict(
    home="data/chainmaind/node0",
    keyring_backend="test",
)

tx_default_args = dict(
    chain_id="chainmaind",
)


def parse_events(rsp):
    return [
        {attr["key"]: attr["value"] for attr in evt["attributes"]}
        for evt in json.loads(rsp["raw_log"])[0]["events"]
    ]


def test_basic():
    addr = wasmd("keys", "show", "community", "-a", **default_args).strip().decode()
    print("addr", addr)
    rsp = json.loads(
        wasmd(
            "tx",
            "wasm",
            "store",
            "target/wasm32-unknown-unknown/release/cw_subscription.wasm",
            "-y",
            from_=addr,
            gas=2000000,
            **default_args,
            **tx_default_args
        )
    )
    assert rsp.get("code", 0) == 0
    code_id = int(parse_events(rsp)[0]["code_id"])
    init_msg = {
        "params": {
            "required_deposit_plan": [{"amount": "1000", "denom": "ucosm"}],
            "required_deposit_subscription": [{"amount": "1000", "denom": "ucosm"}],
        },
    }
    rsp = json.loads(
        wasmd(
            "tx",
            "wasm",
            "instantiate",
            code_id,
            json.dumps(init_msg),
            "-y",
            label="subscription",
            admin=addr,
            from_=addr,
            **default_args,
            **tx_default_args
        )
    )
    assert rsp.get("code", 0) == 0
    _ = parse_events(rsp)[0]["contract_address"]
