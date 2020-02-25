namespace ImageSort.Actions
{
    public interface IReversibleAction
    {
        void Act();
        void Revert();

        string DisplayName { get; }
    }
}
