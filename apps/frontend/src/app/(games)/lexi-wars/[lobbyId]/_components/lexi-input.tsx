import { cn } from "@/lib/utils";

interface LexiInputProps {
	word: string;
	inputRef: React.RefObject<HTMLInputElement | null>;
}

export default function LexiInput({ word, inputRef }: LexiInputProps) {
	return (
		<div
			className={cn(
				"file:text-foreground placeholder:text-muted-foreground selection:bg-primary selection:text-primary-foreground dark:bg-input/30 border-input flex h-9 w-full min-w-0 rounded-md border bg-transparent px-3 py-1 text-base shadow-xs transition-[color,box-shadow] outline-none file:inline-flex file:h-7 file:border-0 file:bg-transparent file:text-sm file:font-medium disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50 md:text-sm",
				"focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[3px]",
				"aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive"
			)}
			onClick={() => inputRef.current?.focus()}
		>
			{word.length === 0 ? (
				<span className="text-muted-foreground">
					Type your word here...
				</span>
			) : (
				word
			)}
			<span className="animate-blink text-primary"></span>
		</div>
	);
}
