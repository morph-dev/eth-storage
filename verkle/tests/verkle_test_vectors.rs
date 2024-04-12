#[cfg(test)]
mod verkle_test_vectors {
    // Tests from https://github.com/jsign/verkle-test-vectors

    use alloy_primitives::{hex::FromHex, Address, B256, U256};
    use anyhow::Result;
    use banderwagon::{CanonicalDeserialize, Element, Fr};
    use db::memory_db::MemoryDb;
    use verkle::{account::AccountStorageLayout, Trie, TrieValue};

    fn element_to_fr(str: &str) -> Fr {
        Element::deserialize_compressed(hex::decode(str).unwrap().as_slice())
            .unwrap()
            .map_to_scalar_field()
    }

    #[test]
    fn test_001_eoa_insert() -> Result<()> {
        let mut trie = Trie::new(Box::new(MemoryDb::new()));
        let address =
            Address::from_slice(&hex::decode("3b7c4c2b2b25239e58f8e67509b32edb5bbf293c")?);

        trie.create_eoa(address, U256::from(8832), 32)?;

        assert_eq!(
            trie.root_hash_commitment()?,
            element_to_fr("43ca08d7ec0f76747e5615e00792c84de5d0ac2753fdef2315a6106d5917b332"),
        );

        Ok(())
    }

    #[test]
    fn test_002_sc_insert() -> Result<()> {
        const CODE: &str = "608060405234801561001057600080fd5b50600436106101b95760003560e01c80636fcfff45116100f9578063b4b5ea5711610097578063dd62ed3e11610071578063dd62ed3e1461036b578063e7a324dc1461037e578063f1127ed814610386578063fca3b5aa146103a7576101b9565b8063b4b5ea5714610332578063c3cda52014610345578063d505accf14610358576101b9565b8063782d6fe1116100d3578063782d6fe1146102e45780637ecebe001461030457806395d89b4114610317578063a9059cbb1461031f576101b9565b80636fcfff45146102b657806370a08231146102c957806376c71ca1146102dc576101b9565b806330adf81f1161016657806340c10f191161014057806340c10f1914610266578063587cde1e1461027b5780635c11d62f1461028e5780635c19a95c146102a3576101b9565b806330adf81f1461024157806330b36cef14610249578063313ce56714610251576101b9565b806318160ddd1161019757806318160ddd1461021157806320606b701461022657806323b872dd1461022e576101b9565b806306fdde03146101be57806307546172146101dc578063095ea7b3146101f1575b600080fd5b6101c66103ba565b6040516101d39190612c72565b60405180910390f35b6101e46103f3565b6040516101d39190612b45565b6102046101ff3660046122aa565b61040f565b6040516101d39190612b6e565b610219610534565b6040516101d39190612b7c565b61021961053a565b61020461023c3660046121c1565b610551565b6102196106f5565b610219610701565b610259610707565b6040516101d39190612dac565b6102796102743660046122aa565b61070c565b005b6101e4610289366004612161565b6109fc565b610296610a24565b6040516101d39190612d83565b6102796102b1366004612161565b610a2c565b6102966102c4366004612161565b610a39565b6102196102d7366004612161565b610a51565b610259610a87565b6102f76102f23660046122aa565b610a8c565b6040516101d39190612dc8565b610219610312366004612161565b610d6e565b6101c6610d80565b61020461032d3660046122aa565b610db9565b6102f7610340366004612161565b610df5565b6102796103533660046122da565b610ea3565b61027961036636600461220e565b611128565b610219610379366004612187565b61155d565b6102196115a3565b610399610394366004612361565b6115af565b6040516101d3929190612d91565b6102796103b5366004612161565b6115ea565b6040518060400160405280600781526020017f556e69737761700000000000000000000000000000000000000000000000000081525081565b60015473ffffffffffffffffffffffffffffffffffffffff1681565b6000807fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff83141561046157507fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff610486565b61048383604051806060016040528060248152602001613082602491396116d6565b90505b33600081815260036020908152604080832073ffffffffffffffffffffffffffffffffffffffff891680855292529182902080547fffffffffffffffffffffffffffffffffffffffff000000000000000000000000166bffffffffffffffffffffffff861617905590519091907f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b92590610520908590612dba565b60405180910390a360019150505b92915050565b60005481565b60405161054690612b2f565b604051809103902081565b73ffffffffffffffffffffffffffffffffffffffff831660009081526003602090815260408083203380855290835281842054825160608101909352602480845291936bffffffffffffffffffffffff9091169285926105bb9288929190613082908301396116d6565b90508673ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff161415801561060757506bffffffffffffffffffffffff82811614155b156106db57600061063183836040518060600160405280603c8152602001612f02603c9139611728565b73ffffffffffffffffffffffffffffffffffffffff8981166000818152600360209081526040808320948a16808452949091529081902080547fffffffffffffffffffffffffffffffffffffffff000000000000000000000000166bffffffffffffffffffffffff86161790555192935090917f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925906106d1908590612dba565b60405180910390a3505b6106e687878361178b565b600193505050505b9392505050565b60405161054690612b24565b60025481565b601281565b60015473ffffffffffffffffffffffffffffffffffffffff163314610766576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161075d90612d13565b60405180910390fd5b6002544210156107a2576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161075d90612cd3565b73ffffffffffffffffffffffffffffffffffffffff82166107ef576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161075d90612cc3565b6107fd426301e133806119f2565b600281905550600061082782604051806060016040528060218152602001613061602191396116d6565b905061084361083c600054600260ff16611a31565b6064611a85565b816bffffffffffffffffffffffff16111561088a576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161075d90612d03565b6108c56108a7600054836bffffffffffffffffffffffff166119f2565b604051806060016040528060268152602001612f3e602691396116d6565b6bffffffffffffffffffffffff908116600090815573ffffffffffffffffffffffffffffffffffffffff85168152600460209081526040918290205482516060810190935260248084526109299491909116928592909190612ede90830139611ac7565b73ffffffffffffffffffffffffffffffffffffffff841660008181526004602052604080822080547fffffffffffffffffffffffffffffffffffffffff000000000000000000000000166bffffffffffffffffffffffff959095169490941790935591519091907fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef906109bd908590612dba565b60405180910390a373ffffffffffffffffffffffffffffffffffffffff8084166000908152600560205260408120546109f7921683611b22565b505050565b60056020526000908152604090205473ffffffffffffffffffffffffffffffffffffffff1681565b6301e1338081565b610a363382611d69565b50565b60076020526000908152604090205463ffffffff1681565b73ffffffffffffffffffffffffffffffffffffffff166000908152600460205260409020546bffffffffffffffffffffffff1690565b600281565b6000438210610ac7576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161075d90612cb3565b73ffffffffffffffffffffffffffffffffffffffff831660009081526007602052604090205463ffffffff1680610b0257600091505061052e565b73ffffffffffffffffffffffffffffffffffffffff8416600090815260066020908152604080832063ffffffff7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff860181168552925290912054168310610bda5773ffffffffffffffffffffffffffffffffffffffff841660009081526006602090815260408083207fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff9490940163ffffffff168352929052205464010000000090046bffffffffffffffffffffffff16905061052e565b73ffffffffffffffffffffffffffffffffffffffff8416600090815260066020908152604080832083805290915290205463ffffffff16831015610c2257600091505061052e565b60007fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff82015b8163ffffffff168163ffffffff161115610d1657600282820363ffffffff16048103610c7261211e565b5073ffffffffffffffffffffffffffffffffffffffff8716600090815260066020908152604080832063ffffffff8581168552908352928190208151808301909252549283168082526401000000009093046bffffffffffffffffffffffff169181019190915290871415610cf15760200151945061052e9350505050565b805163ffffffff16871115610d0857819350610d0f565b6001820392505b5050610c48565b5073ffffffffffffffffffffffffffffffffffffffff8516600090815260066020908152604080832063ffffffff909416835292905220546bffffffffffffffffffffffff6401000000009091041691505092915050565b60086020526000908152604090205481565b6040518060400160405280600381526020017f554e49000000000000000000000000000000000000000000000000000000000081525081565b600080610dde8360405180606001604052806025815260200161303c602591396116d6565b9050610deb33858361178b565b5060019392505050565b73ffffffffffffffffffffffffffffffffffffffff811660009081526007602052604081205463ffffffff1680610e2d5760006106ee565b73ffffffffffffffffffffffffffffffffffffffff831660009081526006602090815260408083207fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff850163ffffffff16845290915290205464010000000090046bffffffffffffffffffffffff169392505050565b6000604051610eb190612b2f565b60408051918290038220828201909152600782527f556e6973776170000000000000000000000000000000000000000000000000006020909201919091527f99c45e8ee5dde061ced9c812089094fbd28a020e7e37f2851198887e5ca64985610f18611e1d565b30604051602001610f2c9493929190612c22565b6040516020818303038152906040528051906020012090506000604051610f5290612b3a565b604051908190038120610f6d918a908a908a90602001612be4565b60405160208183030381529060405280519060200120905060008282604051602001610f9a929190612af3565b604051602081830303815290604052805190602001209050600060018288888860405160008152602001604052604051610fd79493929190612c57565b6020604051602081039080840390855afa158015610ff9573d6000803e3d6000fd5b50506040517fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe0015191505073ffffffffffffffffffffffffffffffffffffffff8116611071576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161075d90612ca3565b73ffffffffffffffffffffffffffffffffffffffff8116600090815260086020526040902080546001810190915589146110d7576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161075d90612d33565b87421115611111576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161075d90612c83565b61111b818b611d69565b505050505b505050505050565b60007fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff86141561117957507fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff61119e565b61119b86604051806060016040528060238152602001612f99602391396116d6565b90505b60006040516111ac90612b2f565b60408051918290038220828201909152600782527f556e6973776170000000000000000000000000000000000000000000000000006020909201919091527f99c45e8ee5dde061ced9c812089094fbd28a020e7e37f2851198887e5ca64985611213611e1d565b306040516020016112279493929190612c22565b604051602081830303815290604052805190602001209050600060405161124d90612b24565b6040805191829003822073ffffffffffffffffffffffffffffffffffffffff8d1660009081526008602090815292902080546001810190915561129c9391928e928e928e9290918e9101612b8a565b604051602081830303815290604052805190602001209050600082826040516020016112c9929190612af3565b6040516020818303038152906040528051906020012090506000600182898989604051600081526020016040526040516113069493929190612c57565b6020604051602081039080840390855afa158015611328573d6000803e3d6000fd5b50506040517fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe0015191505073ffffffffffffffffffffffffffffffffffffffff81166113a0576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161075d90612d63565b8b73ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff1614611405576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161075d90612d73565b8842111561143f576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161075d90612d53565b84600360008e73ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060008d73ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060006101000a8154816bffffffffffffffffffffffff02191690836bffffffffffffffffffffffff1602179055508a73ffffffffffffffffffffffffffffffffffffffff168c73ffffffffffffffffffffffffffffffffffffffff167f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925876040516115479190612dba565b60405180910390a3505050505050505050505050565b73ffffffffffffffffffffffffffffffffffffffff91821660009081526003602090815260408083209390941682529190915220546bffffffffffffffffffffffff1690565b60405161054690612b3a565b600660209081526000928352604080842090915290825290205463ffffffff81169064010000000090046bffffffffffffffffffffffff1682565b60015473ffffffffffffffffffffffffffffffffffffffff16331461163b576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161075d90612d23565b6001546040517f3b0007eb941cf645526cbb3a4fdaecda9d28ce4843167d9263b536a1f1edc0f6916116879173ffffffffffffffffffffffffffffffffffffffff909116908490612b53565b60405180910390a1600180547fffffffffffffffffffffffff00000000000000000000000000000000000000001673ffffffffffffffffffffffffffffffffffffffff92909216919091179055565b6000816c010000000000000000000000008410611720576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161075d9190612c72565b509192915050565b6000836bffffffffffffffffffffffff16836bffffffffffffffffffffffff1611158290611783576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161075d9190612c72565b505050900390565b73ffffffffffffffffffffffffffffffffffffffff83166117d8576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161075d90612d43565b73ffffffffffffffffffffffffffffffffffffffff8216611825576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161075d90612ce3565b73ffffffffffffffffffffffffffffffffffffffff8316600090815260046020908152604091829020548251606081019093526035808452611882936bffffffffffffffffffffffff9092169285929190612f6490830139611728565b73ffffffffffffffffffffffffffffffffffffffff848116600090815260046020908152604080832080547fffffffffffffffffffffffffffffffffffffffff000000000000000000000000166bffffffffffffffffffffffff96871617905592861682529082902054825160608101909352602f80845261191494919091169285929091906130a690830139611ac7565b73ffffffffffffffffffffffffffffffffffffffff8381166000818152600460205260409081902080547fffffffffffffffffffffffffffffffffffffffff000000000000000000000000166bffffffffffffffffffffffff95909516949094179093559151908516907fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef906119ab908590612dba565b60405180910390a373ffffffffffffffffffffffffffffffffffffffff8084166000908152600560205260408082205485841683529120546109f792918216911683611b22565b6000828201838110156106ee576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161075d90612c93565b600082611a405750600061052e565b82820282848281611a4d57fe5b04146106ee576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161075d90612cf3565b60006106ee83836040518060400160405280601a81526020017f536166654d6174683a206469766973696f6e206279207a65726f000000000000815250611e21565b6000838301826bffffffffffffffffffffffff8087169083161015611b19576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161075d9190612c72565b50949350505050565b8173ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff1614158015611b6c57506000816bffffffffffffffffffffffff16115b156109f75773ffffffffffffffffffffffffffffffffffffffff831615611c6f5773ffffffffffffffffffffffffffffffffffffffff831660009081526007602052604081205463ffffffff169081611bc6576000611c36565b73ffffffffffffffffffffffffffffffffffffffff851660009081526006602090815260408083207fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff860163ffffffff16845290915290205464010000000090046bffffffffffffffffffffffff165b90506000611c5d828560405180606001604052806027815260200161301560279139611728565b9050611c6b86848484611e72565b5050505b73ffffffffffffffffffffffffffffffffffffffff8216156109f75773ffffffffffffffffffffffffffffffffffffffff821660009081526007602052604081205463ffffffff169081611cc4576000611d34565b73ffffffffffffffffffffffffffffffffffffffff841660009081526006602090815260408083207fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff860163ffffffff16845290915290205464010000000090046bffffffffffffffffffffffff165b90506000611d5b8285604051806060016040528060268152602001612fbc60269139611ac7565b905061112085848484611e72565b73ffffffffffffffffffffffffffffffffffffffff808316600081815260056020818152604080842080546004845282862054949093528787167fffffffffffffffffffffffff000000000000000000000000000000000000000084168117909155905191909516946bffffffffffffffffffffffff9092169391928592917f3134e8a2e6d97e929a7e54011ea5485d7d196dd5f0ba4d4ef95803e8e3fc257f9190a4611e17828483611b22565b50505050565b4690565b60008183611e5c576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161075d9190612c72565b506000838581611e6857fe5b0495945050505050565b6000611e9643604051806060016040528060338152602001612fe2603391396120dc565b905060008463ffffffff16118015611f0a575073ffffffffffffffffffffffffffffffffffffffff8516600090815260066020908152604080832063ffffffff7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff8901811685529252909120548282169116145b15611fa95773ffffffffffffffffffffffffffffffffffffffff851660009081526006602090815260408083207fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff880163ffffffff168452909152902080547fffffffffffffffffffffffffffffffff000000000000000000000000ffffffff166401000000006bffffffffffffffffffffffff851602179055612085565b60408051808201825263ffffffff80841682526bffffffffffffffffffffffff808616602080850191825273ffffffffffffffffffffffffffffffffffffffff8b166000818152600683528781208c871682528352878120965187549451909516640100000000027fffffffffffffffffffffffffffffffff000000000000000000000000ffffffff9587167fffffffffffffffffffffffffffffffffffffffffffffffffffffffff00000000958616179590951694909417909555938252600790935292909220805460018801909316929091169190911790555b8473ffffffffffffffffffffffffffffffffffffffff167fdec2bacdd2f05b59de34da9b523dff8be42e5e38e818c82fdb0bae774387a72484846040516120cd929190612dd6565b60405180910390a25050505050565b6000816401000000008410611720576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161075d9190612c72565b604080518082019091526000808252602082015290565b803561052e81612eae565b803561052e81612ec2565b803561052e81612ecb565b803561052e81612ed4565b60006020828403121561217357600080fd5b600061217f8484612135565b949350505050565b6000806040838503121561219a57600080fd5b60006121a68585612135565b92505060206121b785828601612135565b9150509250929050565b6000806000606084860312156121d657600080fd5b60006121e28686612135565b93505060206121f386828701612135565b925050604061220486828701612140565b9150509250925092565b600080600080600080600060e0888a03121561222957600080fd5b60006122358a8a612135565b97505060206122468a828b01612135565b96505060406122578a828b01612140565b95505060606122688a828b01612140565b94505060806122798a828b01612156565b93505060a061228a8a828b01612140565b92505060c061229b8a828b01612140565b91505092959891949750929550565b600080604083850312156122bd57600080fd5b60006122c98585612135565b92505060206121b785828601612140565b60008060008060008060c087890312156122f357600080fd5b60006122ff8989612135565b965050602061231089828a01612140565b955050604061232189828a01612140565b945050606061233289828a01612156565b935050608061234389828a01612140565b92505060a061235489828a01612140565b9150509295509295509295565b6000806040838503121561237457600080fd5b60006123808585612135565b92505060206121b78582860161214b565b61239a81612e03565b82525050565b61239a81612e0e565b61239a81612e13565b61239a6123be82612e13565b612e13565b60006123ce82612df1565b6123d88185612df5565b93506123e8818560208601612e5a565b6123f181612e86565b9093019392505050565b6000612408602583612df5565b7f556e693a3a64656c656761746542795369673a207369676e617475726520657881527f7069726564000000000000000000000000000000000000000000000000000000602082015260400192915050565b6000612467600283612dfe565b7f1901000000000000000000000000000000000000000000000000000000000000815260020192915050565b60006124a0601b83612df5565b7f536166654d6174683a206164646974696f6e206f766572666c6f770000000000815260200192915050565b60006124d9602583612df5565b7f556e693a3a64656c656761746542795369673a20696e76616c6964207369676e81527f6174757265000000000000000000000000000000000000000000000000000000602082015260400192915050565b6000612538602683612df5565b7f556e693a3a6765745072696f72566f7465733a206e6f7420796574206465746581527f726d696e65640000000000000000000000000000000000000000000000000000602082015260400192915050565b6000612597602e83612df5565b7f556e693a3a6d696e743a2063616e6e6f74207472616e7366657220746f20746881527f65207a65726f2061646472657373000000000000000000000000000000000000602082015260400192915050565b60006125f6605283612dfe565b7f5065726d69742861646472657373206f776e65722c616464726573732073706581527f6e6465722c75696e743235362076616c75652c75696e74323536206e6f6e636560208201527f2c75696e7432353620646561646c696e65290000000000000000000000000000604082015260520192915050565b600061267b602283612df5565b7f556e693a3a6d696e743a206d696e74696e67206e6f7420616c6c6f776564207981527f6574000000000000000000000000000000000000000000000000000000000000602082015260400192915050565b60006126da604383612dfe565b7f454950373132446f6d61696e28737472696e67206e616d652c75696e7432353681527f20636861696e49642c6164647265737320766572696679696e67436f6e74726160208201527f6374290000000000000000000000000000000000000000000000000000000000604082015260430192915050565b600061275f603983612df5565b7f556e693a3a5f7472616e73666572546f6b656e733a2063616e6e6f742074726181527f6e7366657220746f20746865207a65726f206164647265737300000000000000602082015260400192915050565b60006127be602183612df5565b7f536166654d6174683a206d756c7469706c69636174696f6e206f766572666c6f81527f7700000000000000000000000000000000000000000000000000000000000000602082015260400192915050565b600061281d601c83612df5565b7f556e693a3a6d696e743a206578636565646564206d696e742063617000000000815260200192915050565b6000612856602383612df5565b7f556e693a3a6d696e743a206f6e6c7920746865206d696e7465722063616e206d81527f696e740000000000000000000000000000000000000000000000000000000000602082015260400192915050565b60006128b5603d83612df5565b7f556e693a3a7365744d696e7465723a206f6e6c7920746865206d696e7465722081527f63616e206368616e676520746865206d696e7465722061646472657373000000602082015260400192915050565b6000612914602183612df5565b7f556e693a3a64656c656761746542795369673a20696e76616c6964206e6f6e6381527f6500000000000000000000000000000000000000000000000000000000000000602082015260400192915050565b6000612973603b83612df5565b7f556e693a3a5f7472616e73666572546f6b656e733a2063616e6e6f742074726181527f6e736665722066726f6d20746865207a65726f20616464726573730000000000602082015260400192915050565b60006129d2601e83612df5565b7f556e693a3a7065726d69743a207369676e617475726520657870697265640000815260200192915050565b6000612a0b603a83612dfe565b7f44656c65676174696f6e28616464726573732064656c6567617465652c75696e81527f74323536206e6f6e63652c75696e7432353620657870697279290000000000006020820152603a0192915050565b6000612a6a601e83612df5565b7f556e693a3a7065726d69743a20696e76616c6964207369676e61747572650000815260200192915050565b6000612aa3601983612df5565b7f556e693a3a7065726d69743a20756e617574686f72697a656400000000000000815260200192915050565b61239a81612e2f565b61239a81612e38565b61239a81612e4f565b61239a81612e3e565b6000612afe8261245a565b9150612b0a82856123b2565b602082019150612b1a82846123b2565b5060200192915050565b600061052e826125e9565b600061052e826126cd565b600061052e826129fe565b6020810161052e8284612391565b60408101612b618285612391565b6106ee6020830184612391565b6020810161052e82846123a0565b6020810161052e82846123a9565b60c08101612b9882896123a9565b612ba56020830188612391565b612bb26040830187612391565b612bbf60608301866123a9565b612bcc60808301856123a9565b612bd960a08301846123a9565b979650505050505050565b60808101612bf282876123a9565b612bff6020830186612391565b612c0c60408301856123a9565b612c1960608301846123a9565b95945050505050565b60808101612c3082876123a9565b612c3d60208301866123a9565b612c4a60408301856123a9565b612c196060830184612391565b60808101612c6582876123a9565b612bff6020830186612ad8565b602080825281016106ee81846123c3565b6020808252810161052e816123fb565b6020808252810161052e81612493565b6020808252810161052e816124cc565b6020808252810161052e8161252b565b6020808252810161052e8161258a565b6020808252810161052e8161266e565b6020808252810161052e81612752565b6020808252810161052e816127b1565b6020808252810161052e81612810565b6020808252810161052e81612849565b6020808252810161052e816128a8565b6020808252810161052e81612907565b6020808252810161052e81612966565b6020808252810161052e816129c5565b6020808252810161052e81612a5d565b6020808252810161052e81612a96565b6020810161052e8284612acf565b60408101612d9f8285612acf565b6106ee6020830184612aea565b6020810161052e8284612ad8565b6020810161052e8284612ae1565b6020810161052e8284612aea565b60408101612de48285612ae1565b6106ee6020830184612ae1565b5190565b90815260200190565b919050565b600061052e82612e16565b151590565b90565b73ffffffffffffffffffffffffffffffffffffffff1690565b63ffffffff1690565b60ff1690565b6bffffffffffffffffffffffff1690565b600061052e82612e3e565b60005b83811015612e75578181015183820152602001612e5d565b83811115611e175750506000910152565b601f017fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe01690565b612eb781612e03565b8114610a3657600080fd5b612eb781612e13565b612eb781612e2f565b612eb781612e3856fe556e693a3a6d696e743a207472616e7366657220616d6f756e74206f766572666c6f7773556e693a3a7472616e7366657246726f6d3a207472616e7366657220616d6f756e742065786365656473207370656e64657220616c6c6f77616e6365556e693a3a6d696e743a20746f74616c537570706c7920657863656564732039362062697473556e693a3a5f7472616e73666572546f6b656e733a207472616e7366657220616d6f756e7420657863656564732062616c616e6365556e693a3a7065726d69743a20616d6f756e7420657863656564732039362062697473556e693a3a5f6d6f7665566f7465733a20766f746520616d6f756e74206f766572666c6f7773556e693a3a5f7772697465436865636b706f696e743a20626c6f636b206e756d62657220657863656564732033322062697473556e693a3a5f6d6f7665566f7465733a20766f746520616d6f756e7420756e646572666c6f7773556e693a3a7472616e736665723a20616d6f756e7420657863656564732039362062697473556e693a3a6d696e743a20616d6f756e7420657863656564732039362062697473556e693a3a617070726f76653a20616d6f756e7420657863656564732039362062697473556e693a3a5f7472616e73666572546f6b656e733a207472616e7366657220616d6f756e74206f766572666c6f7773a365627a7a7231582068d42e51eae03f461fed8d2db904ba521e900521e0a2199b4a57d733502ad3da6c6578706572696d656e74616cf564736f6c63430005100040";

        let mut trie = Trie::new(Box::new(MemoryDb::new()));
        let address =
            Address::from_slice(&hex::decode("1f9840a85d5aF5bf1D1762F925BDADdC4201F984")?);

        trie.create_sc(address, U256::from(100), 21, hex::decode(CODE)?)?;

        assert_eq!(
            trie.root_hash_commitment()?,
            element_to_fr("70d33ec88ca3efcca85510f1511b5bd063c6812ca1f9ec70e15a4d8b6637d9a8"),
        );

        Ok(())
    }

    #[test]
    fn test_003_tree_mutation() -> Result<()> {
        let mut trie = Trie::new(Box::new(MemoryDb::new()));

        let key_values = vec![
            (
                "0x0000000000000000000000000000000000000000000000000000000000000000",
                "42",
            ),
            (
                "0x0100000000000000000000000000000000000000000000000000000000000000",
                "10",
            ),
            (
                "0x0100000000000000000000000000000000000000000000000000000000000001",
                "11",
            ),
            (
                "0x02000000000000000000000000000000000000000000000000000000000000FF",
                "20",
            ),
            (
                "0x0300000000000000000000000000000000000000000000000000000000000000",
                "30",
            ),
            (
                "0x0300000000000000000000000000000000000000000000000000000000000080",
                "31",
            ),
            (
                "0x0400000000000000000000000000000000000000000000000000000000000000",
                "44",
            ),
            (
                "0x0401000000000000000000000000000000000000000000000000000000000000",
                "4040",
            ),
            (
                "0x0500000000000000000000000000000000000000000000000000000000000000",
                "50",
            ),
            (
                "0x05000000000000000000000000000000000000000000000000000000000001FF",
                "50000010",
            ),
            (
                "0x0401000000000000000000000000000000000000000000000000000000000000",
                "2000",
            ),
        ];
        for (key, value) in key_values.into_iter() {
            trie.insert(
                B256::from_hex(key)?.into(),
                TrieValue::from_le_slice(&hex::decode(value)?),
            )?;
        }

        assert_eq!(
            trie.root_hash_commitment()?,
            element_to_fr("65b22f15dfba6292ed91332d9ff568d77d51aa072d48c53d18343905c74a719b"),
        );

        Ok(())
    }

    #[test]
    fn test_004_storageslot_insert() -> Result<()> {
        let mut trie = Trie::new(Box::new(MemoryDb::new()));
        let address =
            Address::from_slice(&hex::decode("3b7c4c2b2b25239e58f8e67509b32edb5bbf293c")?);
        let storage_slots = vec![0, 1, 31, 32, 63, 64, 100, 1000, 10001];
        let value = TrieValue::from_le_bytes(B256::left_padding_from(b"elephant").0);

        let storage_layout = AccountStorageLayout::new(address);
        for storage_slot in storage_slots {
            trie.insert(
                storage_layout.storage_slot_key(U256::from(storage_slot)),
                value,
            )?;
        }

        assert_eq!(
            trie.root_hash_commitment()?,
            element_to_fr("5f9b9b718f3658151bdc58e594a951787d0307467b13ea4d1cda411428216a1e"),
        );

        Ok(())
    }
}
