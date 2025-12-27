import { NextResponse } from "next/server";
import { getAuthenticatedUserId } from "@/lib/auth/jwt";

export async function GET() {
	const userId = await getAuthenticatedUserId();

	if (!userId) {
		return NextResponse.json({ authenticated: false, userId: null });
	}

	return NextResponse.json({ authenticated: true, userId });
}
