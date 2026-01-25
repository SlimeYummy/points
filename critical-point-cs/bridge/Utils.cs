using MessagePack;
using MessagePack.Formatters;
using MessagePack.Resolvers;

namespace CriticalPoint {

    internal class Static {
        private static readonly Static _instance = new Static();

        private MessagePackSerializerOptions _msgPackOpts;

        private Static() {
            var resolver = CompositeResolver.Create(
                new IMessagePackFormatter[] {
                    new TmplIDFormatter(),
                    new TmplIDLevelFormatter(),
                    new TmplIDPlusFormatter(),
                    new Vec2Formatter(),
                    new Vec3Formatter(),
                    new Vec3AFormatter(),
                    new Vec4Formatter(),
                    new QuatFormatter(),
                    new Mat4Formatter(),
                },
                new IFormatterResolver[] { StandardResolver.Instance }
            );
            _msgPackOpts = MessagePackSerializerOptions.Standard.WithResolver(resolver);
        }

        public static MessagePackSerializerOptions MsgPackOpts => _instance._msgPackOpts;
    }
}
