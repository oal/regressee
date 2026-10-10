// SPDX-License-Identifier: MPL-2.0
// Maintained product v2 boundary. Capture repeats immutable profile validation.
export const VERSION = 2;
export const MAX_FRAME = 1024 * 1024;
export const COMMANDS = Object.freeze(['Handshake', 'SetupLogin', 'OpenLogin', 'ConfirmLogin', 'InstallSession', 'DiscoverAuthenticated', 'DiscoverPage', 'FetchRobots', 'CaptureTarget', 'Cancel', 'Shutdown']);
const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/;
const nil = '00000000-0000-0000-0000-000000000000';
const uuid = value => typeof value === 'string' && UUID.test(value) && value !== nil;
const object = value => value !== null && typeof value === 'object' && !Array.isArray(value);
const exactKeys = (value, keys) => object(value) && Object.keys(value).length === keys.length && keys.every(key => Object.hasOwn(value, key));
const address = value => {
  if (typeof value !== 'string' || Buffer.byteLength(value) > 4096 || /[\s\\]/.test(value) || !/^https?:\/\//i.test(value)) return false;
  try { const url = new URL(value); return ['https:', 'http:'].includes(url.protocol) && !url.username && !url.password; } catch { return false; }
};
// Shared with Rust: all three IDs must be non-nil. Only CaptureTarget may
// explicitly use null for unauthenticated capture; SetupLogin requires a reference.
const session = value => exactKeys(value, ['id', 'project', 'environment']) && ['id', 'project', 'environment'].every(key => uuid(value[key]));
const origins=value=>Array.isArray(value)&&value.length<=16&&value.every(o=>{if(!address(o))return false;const u=new URL(o);return u.href===u.origin+'/';});
export class ProtocolError extends Error {
  constructor(code) { super(code); this.code = code; }
}
export function encodeEvent(r,type,payload) {
  const frame=JSON.stringify({version:VERSION,request_id:r.request_id,job:r.job,event:payload===undefined?{type}:{type,payload}})+'\n';
  if(Buffer.byteLength(frame)>MAX_FRAME)throw new ProtocolError('PayloadLimit');
  return frame;
}
export function parseRequest(frame) {
  // Frame includes its newline. Do not retain unbounded partial input buffers.
  if (!Buffer.isBuffer(frame) || frame.length > MAX_FRAME || frame.at(-1) !== 10) throw new ProtocolError('PayloadLimit');
  let request;
  try { request = JSON.parse(new TextDecoder('utf-8', {fatal:true}).decode(frame)); } catch { throw new ProtocolError('InvalidRequest'); }
  if (!exactKeys(request, ['version', 'request_id', 'job', 'command']) || request.version !== VERSION) throw new ProtocolError('UnsupportedVersion');
  if (!Number.isSafeInteger(request.request_id) || request.request_id <= 0 || !uuid(request.job) || !object(request.command) || !COMMANDS.includes(request.command.type)) throw new ProtocolError('InvalidRequest');
  const {type, payload} = request.command;
  if (['Handshake', 'Cancel', 'Shutdown'].includes(type)) {
    if (!exactKeys(request.command, ['type'])) throw new ProtocolError('InvalidRequest');
  } else {
    if (!exactKeys(request.command, ['type', 'payload'])) throw new ProtocolError('InvalidRequest');
    if (type === 'SetupLogin' && (!exactKeys(payload, ['session']) || !session(payload.session))) throw new ProtocolError('InvalidRequest');
    if(type==='OpenLogin'&&(!exactKeys(payload,['session','address','representative','origins','identity_origins'])||!session(payload.session)||!address(payload.address)||!address(payload.representative)||!origins(payload.origins)||!origins(payload.identity_origins)))throw new ProtocolError('InvalidRequest');
    if(type==='ConfirmLogin'&&(!exactKeys(payload,['session'])||!session(payload.session)))throw new ProtocolError('InvalidRequest');
    if(type==='InstallSession'&&(!exactKeys(payload,['session','origins','material'])||!session(payload.session)||!origins(payload.origins)||typeof payload.material!=='string'||Buffer.byteLength(payload.material)>512*1024))throw new ProtocolError('InvalidRequest');
    if(type==='DiscoverAuthenticated'&&(!exactKeys(payload,['address','section','session'])||!session(payload.session)||!address(payload.address)||(payload.section!==null&&(typeof payload.section!=='string'||!payload.section.startsWith('/')||payload.section.length>4096))))throw new ProtocolError('InvalidRequest');
    if (type === 'DiscoverPage' && (!exactKeys(payload, ['address','section']) || !address(payload.address) || (payload.section !== null && (typeof payload.section !== 'string' || !payload.section.startsWith('/') || payload.section.length>4096)))) throw new ProtocolError('InvalidRequest');
    if (type === 'FetchRobots' && (!exactKeys(payload, ['address']) || !address(payload.address))) throw new ProtocolError('InvalidRequest');
    if (type === 'CaptureTarget') {
      if (!exactKeys(payload, ['key', 'address', 'profile', 'session']) || !address(payload.address) || !exactKeys(payload.key, ['page', 'profile']) || !uuid(payload.key.page) || !uuid(payload.key.profile) || (payload.session !== null && !session(payload.session))) throw new ProtocolError('InvalidRequest');
      // Full immutable profile validation is repeated in capture.mjs.
      if (!object(payload.profile) || payload.profile.version !== 1 || !uuid(payload.profile.id) || payload.profile.id !== payload.key.profile || !object(payload.profile.runtime) || !object(payload.profile.runtime.Pinned)) throw new ProtocolError('InvalidRequest');
    }
  }
  return request;
}
