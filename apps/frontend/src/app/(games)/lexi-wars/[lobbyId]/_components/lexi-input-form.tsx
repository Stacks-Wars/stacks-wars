import { Input } from "@/components/ui/input";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Loader2 } from "lucide-react";

interface LexiInputFormProps {
	word: string;
	setWord: (word: string) => void;
	handleSubmit: (e?: React.FormEvent) => void;
	isLoading: boolean;
	disabled?: boolean;
}

export default function LexiInputForm({
	word,
	setWord,
	handleSubmit,
	isLoading,
	disabled = false,
}: LexiInputFormProps) {
	const handlePaste = (e: React.ClipboardEvent) => {
		e.preventDefault();
		toast.error("Pasting is not permited!", { position: "top-center" });
	};

	const handleCopy = (e: React.ClipboardEvent) => {
		e.preventDefault();
		toast.error("Copying is not permited!", { position: "top-center" });
	};

	const handleCut = (e: React.ClipboardEvent) => {
		e.preventDefault();
		toast.error("Cutting is not permited!", { position: "top-center" });
	};

	//console.log("isTouchDevice", isTouchDevice);

	return (
		<form
			onSubmit={handleSubmit}
			autoComplete="off"
			className="space-y-3 sm:space-y-4"
		>
			<Input
				type="text"
				placeholder={
					disabled
						? "You are spectating this game"
						: "Type your word here..."
				}
				value={word}
				onChange={(e) => setWord(e.target.value)}
				onPaste={handlePaste}
				onCopy={handleCopy}
				onCut={handleCut}
				className="h-12 text-lg sm:px-4 sm:text-xl"
				inputMode="text"
				autoComplete="off"
				aria-autocomplete="none"
				autoCorrect="off"
				spellCheck={false}
				autoCapitalize="off"
				autoFocus={!disabled}
				disabled={disabled}
				//disabled={!isPlaying || timeLeft === 0}
				//className="absolute opacity-0 pointer-events-none h-0 w-0"
				//aria-hidden={isTouchDevice}
				//readOnly={isTouchDevice}
			/>

			<div className="flex justify-end">
				<Button
					type="submit"
					disabled={isLoading || disabled}
					size="lg"
					className="w-full md:w-fit"
				>
					{isLoading && (
						<Loader2 className="mr-2 h-4 w-4 animate-spin" />
					)}
					{disabled ? "You're Spectating" : "Submit"}
				</Button>
			</div>
		</form>
	);
}
