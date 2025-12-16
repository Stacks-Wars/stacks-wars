const DEV = process.env.NODE_ENV !== "production";
export const DOMAIN_NAME = "stackswars.com";
export const FRONTEND_LOCALHOST_PORT = 4002;
export const FRONTEND_URL = DEV
	? `http://localhost:${FRONTEND_LOCALHOST_PORT}`
	: `https://www.${DOMAIN_NAME}`;

export const BETTERAUTH_URL = DEV
	? `http://localhost:${FRONTEND_LOCALHOST_PORT}`
	: `https://www.${DOMAIN_NAME}`;

export const siteConfig = {
	title: "Stacks Wars",
	authSuccessRedirectUrl: "/games",
};

// UTILS
export const BNS_ONE_API_BASE_URL = "https://api.bns.one/";
