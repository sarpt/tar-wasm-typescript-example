import { parse_tar } from './crate/pkg';
import { example_tar_content_base64 } from './resources/example_tar';

const tar = parse_tar(new TextEncoder().encode(atob(example_tar_content_base64)));
const filenames = tar.get_filenames();
console.log(`[FRONT] filenames`);
console.log(filenames);

const firstFilePayload = tar.get_payload(filenames[0]);
console.log(`[FRONT] filename ${filenames[0]} payload:`);
console.log(new TextDecoder().decode(firstFilePayload));
