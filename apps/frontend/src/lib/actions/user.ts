"use server";

import { cookies } from "next/headers";
import { revalidatePath, revalidateTag } from "next/cache";
import { disconnect } from "@stacks/connect";

export async function connectOrCreateUser(walletAddress: string) {
	const cookieStore = await cookies();
	try {
		const apiRes = await fetch(`${process.env.NEXT_PUBLIC_API_URL}/user`, {
			method: "POST",
			body: JSON.stringify({ wallet_address: walletAddress }),
			headers: {
				"Content-Type": "application/json",
			},
			next: { tags: ["user"] },
		});

		if (!apiRes.ok) {
			throw new Error("Failed to create or login user");
		}

		const token = await apiRes.json();

		cookieStore.set("jwt", token, {
			httpOnly: true,
			secure: true,
			maxAge: 60 * 60 * 24 * 7, // 7 days
			path: "/",
		});

		revalidateTag("user");
		revalidatePath("/");

		return { success: true };
	} catch (err) {
		console.error(err);
		return { success: false, message: "Connection or login failed." };
	}
}

export async function logoutUser() {
	const cookieStore = await cookies();

	try {
		disconnect();
		cookieStore.delete("jwt");
		revalidateTag("user");
		revalidatePath("/");

		return { success: true };
	} catch {
		return { success: false, message: "Logout failed." };
	}
}
