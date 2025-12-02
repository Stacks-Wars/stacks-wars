"use client";

import { Button } from "@/components/ui/button";
import useCopyToClipboard from "@/hooks/useCopy";
import { PUBLIC_BASE_URL } from "@/lib/constants";
import { Share } from "lucide-react";
import { toast } from "sonner";

export default function ShareLinkButton({ lobbyId }: { lobbyId: string }) {
	const [copiedText, copy] = useCopyToClipboard();
	return (
		<Button
			variant={"outline"}
			onClick={() => {
				copy(`${PUBLIC_BASE_URL}lobby/${lobbyId}`);
				if (copiedText) toast.info("Copied lobby link to clipboard");
				else toast.error("Failed to copy lobby link to clipboard");
			}}
		>
			<Share className="h-4 w-4" /> Share
		</Button>
	);
}
