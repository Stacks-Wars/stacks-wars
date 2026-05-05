"use client";
import dynamic from "next/dynamic";

const AuthDialog = dynamic(() =>
	import("../_components/auth-dialog").then((mod) => mod.AuthDialog)
);

export default function SignupModal() {
	return <AuthDialog mode="signup" open={true} />;
}
