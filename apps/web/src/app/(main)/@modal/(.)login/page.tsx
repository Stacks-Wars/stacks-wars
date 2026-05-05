"use client";
import dynamic from "next/dynamic";

const AuthDialog = dynamic(() =>
	import("../_components/auth-dialog").then((mod) => mod.AuthDialog)
);

export default function LoginModal() {
	return <AuthDialog open={true} />;
}
