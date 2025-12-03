//import Link from "next/link";

export default function Footer() {
	return (
		<footer className="bg-primary/30 w-full border-t py-6 md:py-0">
			<div className="flex items-center justify-center gap-4 md:h-24">
				<p className="text-muted-foreground w-fit text-center text-sm leading-loose md:text-left">
					© {new Date().getFullYear()} Stacks Wars. All rights
					reserved.
				</p>
				{/*<div className="flex items-center gap-4">
          <Link
            href="/terms"
            className="text-sm text-muted-foreground underline-offset-4 hover:underline"
          >
            Terms
          </Link>
          <Link
            href="/privacy"
            className="text-sm text-muted-foreground underline-offset-4 hover:underline"
          >
            Privacy
          </Link>
        </div>*/}
			</div>
		</footer>
	);
}
