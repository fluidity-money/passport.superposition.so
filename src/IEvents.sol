// SPDX-License-Identifier: BUSL-1.1
pragma solidity 0.8.20;

interface IEvents {
    event KeyRegistered(
        uint64 indexed offset,
        bytes32 indexed key,
        address indexed owner
    );
}
