import {
	tupleCV,
	uintCV,
	principalCV,
	serializeCV,
	signMessageHashRsv,
	type ContractIdString,
} from "@stacks/transactions";
import { createHash } from "crypto";
import { generateWallet } from "@stacks/wallet-sdk";

const secretKey = process.env.TRUSTED_SECRET_KEY;

const getSignerPrivateKey = async () => {
	if (!secretKey) {
		throw new Error("Secret key is not defined in environment variables");
	}
	const wallet = await generateWallet({ secretKey, password: "" });
	const privateKey = wallet.accounts[0].stxPrivateKey;
	return privateKey;
};

export const generateSignature = async (
	amount: number,
	claimerAddress: string,
	contractAddress: ContractIdString
) => {
	const message = tupleCV({
		amount: uintCV(amount * 1_000_000),
		player: principalCV(claimerAddress),
		contract: principalCV(contractAddress),
	});
	const serialized = serializeCV(message);
	const buffer = Buffer.from(serialized, "hex");
	const hash = createHash("sha256").update(buffer).digest();
	const privateKey = await getSignerPrivateKey();
	return signMessageHashRsv({
		messageHash: hash.toString("hex"),
		privateKey,
	});
};
