import { request } from "@stacks/connect";
import { nanoid } from "nanoid";
import { getStxAddress } from "../wallet";

/**
 * Deploy a Stacks contract
 */
export async function deployStacksContract(params: {
	clarityCode: string;
	tokenName: string;
}) {
	const network = process.env.NEXT_PUBLIC_NETWORK || "testnet";

	const name = `${nanoid(4)}-stacks-wars-${params.tokenName.toLocaleLowerCase()}-vault`;

	const result = await request("stx_deployContract", {
		name,
		clarityCode: params.clarityCode,
		network,
	});

	const contractAddress = `${getStxAddress()}.${name}`;

	return { txid: result.txid, contractAddress };
}
