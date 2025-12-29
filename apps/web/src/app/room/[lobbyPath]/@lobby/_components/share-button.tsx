"use client";

import { Button } from "@/components/ui/button";
import useCopyToClipboard from "@/lib/hooks/useCopy";
import { Share } from "lucide-react";
import { toast } from "sonner";

export default function ShareButton({ lobbyPath }: { lobbyPath: string }) {
	const [copiedtext, copy] = useCopyToClipboard();
	return (
		<Button
			className="rounded-full has-[>svg]:px-5 px-5 py-2.5"
			onClick={() => {
				copy(`/room/${lobbyPath}`);
				toast.info(`Room link copied to clipboard!`);
			}}
		>
			<Share />
			Share
		</Button>
	);
}
