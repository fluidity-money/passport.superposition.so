// SPDX-License-Identifier: MIT
pragma solidity 0.8.20;

import "forge-std/Test.sol";
import "forge-std/console.sol";

import "../src/Proxy.sol";

interface IAF {
    function deployStylusCode(string calldata artifact) external returns (address);
}

contract TestPassportE2E is Test {
    Proxy proxy;

    address solver;
    address applyy;
    address setter;
    address admin;
    address vault;

    function setUp() public {
        admin = IAF(address(vm)).deployStylusCode("admin.passport-superposition-so.wasm");
        applyy = IAF(address(vm)).deployStylusCode("apply.passport-superposition-so.wasm");
        setter = IAF(address(vm)).deployStylusCode("setter.passport-superposition-so.wasm");
        solver = IAF(address(vm)).deployStylusCode("solver.passport-superposition-so.wasm");
        vault = IAF(address(vm)).deployStylusCode("vault.passport-superposition-so.wasm");
        proxy = new Proxy(solver, applyy, setter, admin, vault);
    }

    function testDeploymentsWereOkay() external view {
        assertNotEq(address(0), admin);
        assertNotEq(address(0), applyy);
        assertNotEq(address(0), setter);
        assertNotEq(address(0), solver);
        assertNotEq(address(0), vault);
        assertNotEq(address(0), address(proxy));
    }

    function testBeginAlexOnramp() external {
        console.log(address(this));
        (bool success,) = address(proxy).call(hex"0180eea4968a6f8e8131be42ac7e8b141ead3ff375a39d2be45f5d638863b533ae7c2aa19fa5f5b6f2ad2413f742b5fd7eb21c55deb16fc069a29ea87e1e315c8ad2e2198886ca0562955f2b969e8d172fa989e2ffbd4ebda75717edc82dd46d2f8f5fd6bdc5b0724ec72bbd0681f63ed12bfef7c197ea74fd1008d7c6472cf057a3b82d6c0201a9c1404fe202b15c1e3e96836ebf40e85b5996a7b99dba72731e0f3e427de0a84af2f2380e0e5b2ff665a4d6eed4abfe8a90187ed01d1f84e2309f1ea5ab31cf9a7cfcbfaa4ec27d77e8caa653c937f399439f73745828f4aa4b52eb4e21f43d04021d05b2c1f8140c14374f3cbc71a611cb357a0da1a66ee3119e9cd368");
        assert(success);
    }
}
