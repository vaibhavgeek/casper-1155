This is the proposal for the bounty on creation of an erc 1155 multitoken contract for casper network.

This is still a work under progress and will be updated with more concrete frotend and mock logic. 

The multitoken contract will allow the creation of multiple tokens. These tokens can then be minted either as a funngible asset, only minting once or as a non fungible asset and can be minted multiple times all identified by their __token_id__ as clearly specified in the ERC 1155  (https://github.com/ethereum/EIPs/issues/1155)

## Milestone 1 
- Clone and test the NFT logic locally based on ERC 721 written by Casper Team. [Complete]
- Modify the logic folder (events.rs, lib.rs and tests.rs) to have the proper multi token standard on casper. [Complete] 

Expected Date of Completion: 12.10.21

## Milestone 2 
- Modify the main folder to have the correct logic associated with multi token standard (storage.rs, data.rs, entrypoints.rs, lib.rs)
- Deploy contract on testnet 
- Create documentation based on entrypoints.rs so that the contract can be used by anyone. 

Expeced Date of Completion: 19.10.21

This is a response to the bounty mentioned by casper network, https://gitcoin.co/issue/casper-network/gitcoin-hackathon/14/100026580
