import { auth } from "@stacks-wars/auth";
import { headers } from "next/headers";
import { redirect } from "next/navigation";
import type { JSX } from "react";
import type { Session } from "./definitions/auth";

// HOC for pages that require authentication
export function withAuth<P extends { session: Session }>(
	component: (props: P) => Promise<JSX.Element>
): (props: Omit<P, "session">) => Promise<JSX.Element> {
	return async (props: Omit<P, "session">) => {
		const headersList = await headers();
		const session = await auth.api.getSession({ headers: headersList });

		if (!session) {
			redirect("/");
		}

		return component({ ...props, session } as unknown as P);
	};
}
