import { BNS_ONE_API_BASE_URL } from "@stacks-wars/shared";
import { getPrimaryName } from "bns-v2-sdk";

type BnsZonefileResponse = {
	zonefile?: {
		pfp?: string;
	};
};

export const getBnsAndAvatar = async (walletAddress: string) => {
	try {
		const primaryName = await getPrimaryName({
			address: walletAddress,
			network: "mainnet",
		});
		if (!primaryName) {
			return {
				name: walletAddress,
				avatar: "",
			};
		}
		const bns = `${primaryName.name}.${primaryName.namespace}`;
		const res = await fetch(`${BNS_ONE_API_BASE_URL}zonefile?fqn=${bns}`);

		if (!res.ok) {
			return {
				name: bns,
				avatar: "",
			};
		}

		const data = (await res.json()) as BnsZonefileResponse;
		if (!data.zonefile?.pfp) {
			return {
				name: bns,
				avatar: "",
			};
		}
		return {
			name: bns,
			avatar: data.zonefile.pfp,
		};
	} catch (err) {
		console.error(`BNS lookup failed for ${walletAddress}:`, err);
		return {
			name: walletAddress,
			avatar: "",
		};
	}
};
