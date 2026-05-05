import { nanoid } from "nanoid";

let request: typeof import("@stacks/connect").request;
if (typeof window !== "undefined") {
	request = (await import("@stacks/connect")).request;
}

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

	return { txid: result.txid, name };
}
