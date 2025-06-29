import {connect} from 'puppeteer-real-browser';
import type {ElementHandle, GoToOptions, HTTPRequest} from "rebrowser-puppeteer-core";
import delay from "delay";

function bail(e: any): never {
    if (e.name !== undefined && e.message !== undefined) {
        console.error(`${e.name}: ${e.message}`);
    } else if (e.message !== undefined) {
        console.error(e.message);
    } else {
        console.error(e);
    }

    process.exit(1);
}

const email = process.argv[2] ?? 'default';
const password = process.argv[3] ?? 'default';
const headless = process.argv[4] === 'headless';

const resolvePage = async (url: string, cf: () => boolean, goToOptions: GoToOptions) => {
    const {browser, page} = await connect({
        headless,
        turnstile: true,
        connectOption: {}
    });
    await page.goto(url, goToOptions);
    const isTurnstile: () => Promise<boolean> = async () => await page.evaluate(cf);
    if (!await isTurnstile()) {
        await options(page.waitForSelector('#email', {visible: true, timeout: 5000}));
        return {browser, page}
    }

    let attempts = 30;
    while (await isTurnstile() && attempts > 0) {
        await delay(1000);
        attempts--;
    }

    const isTurnstiled = await isTurnstile();
    return isTurnstiled ? null : {browser, page}
}

function unwrap<T>(t: T | null): T {
    return t ?? bail(Error(`Unwrap on null`))
}

async function type_input<T extends Node>(element: ElementHandle<T>, input: string) {
    await element.focus()
    await element.type(input, {delay: 120})
}

async function options<T>(t: Promise<T>): Promise<Awaited<T> | null> {
    try {
        return await t;
    } catch (e) {
        return null
    }
}

async function main() {
    const {page} = await resolvePage(
        'https://securitytrails.com/app/account',
        () => document.getElementById('challenge-success-text')?.textContent?.toLowerCase() === 'verification successful',
        {referer: 'https://www.google.com', waitUntil: 'networkidle0'}
    )
        .then(unwrap);

    await delay(1000);
    const emailInput = await page.waitForSelector('#email').then(unwrap)
    await type_input(emailInput, email);
    await delay(200);

    const passwordInput = await page.waitForSelector('#password').then(unwrap)
    await type_input(passwordInput, password);
    await delay(300);
    await type_input(passwordInput, "\r\n");
    await delay(3000);

    const [dashboard, invalid] = await Promise.all([
        options(page.waitForSelector('button[name="account-menu"]', {visible: true, timeout: 1000})),
        options(page.waitForSelector('p.text-danger.text-center', {visible: true, timeout: 1000}))
    ]);
    if (dashboard !== null) {
        await delay(3000);
        let fetchRequest: HTTPRequest = undefined!;
        page.on('request', (req) => {
            if (req.url().includes('/_next/data/')) {
                fetchRequest = req;
            }
        });

        (await page.waitForSelector('button[name="toggle-search"]', {visible: true}))?.click();
        await delay(500);
        const searchInput = await page.waitForSelector('#search', {visible: true}).then(unwrap);
        searchInput.click();
        await type_input(searchInput, '\r\n');

        while (!fetchRequest?.url().includes('/_next/data/')) {
            await delay(100);
        }

        const output = {
            success: true,
            message: 'New session acquired',
            cookies: await page.cookies(fetchRequest.url()),
            headers: fetchRequest.headers(),
            unique: fetchRequest.url().split('/')[5] ?? undefined,
        };

        console.log(JSON.stringify(output, null, 2));
        process.exit(0);
    } else if (invalid !== null) {
        bail(Error('Security Trails Invalid credentials'));
    } else {
        bail(Error('Failed to login, no dashboard or invalid credentials found'));
    }
}

await main();
