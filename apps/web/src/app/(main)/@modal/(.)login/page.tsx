"use client";
import { useRouter } from "next/navigation";
import { AuthDialog } from "../_components/auth-dialog";

export default function LoginModal() {
	const router = useRouter();

	return <AuthDialog open={true} onOpenChange={() => router.back()} />;
}
